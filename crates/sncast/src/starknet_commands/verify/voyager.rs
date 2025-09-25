use anyhow::{Context, Result, anyhow};
use camino::{Utf8Path, Utf8PathBuf};
use foundry_ui::{UI, components::warning::WarningMessage};
use itertools::Itertools;
use reqwest::{self, StatusCode};
use scarb_api::metadata::MetadataCommand;
use scarb_metadata::{Metadata, PackageMetadata};
use serde::Serialize;
use sncast::Network;
use sncast::{helpers::scarb_utils, response::verify::VerifyResponse};
use starknet::{
    core::types::{BlockId, BlockTag},
    providers::{
        Provider,
        jsonrpc::{HttpTransport, JsonRpcClient},
    },
};
use starknet_types_core::felt::Felt;
use std::{collections::HashMap, env, ffi::OsStr, fs, path::PathBuf};
use url::Url;
use walkdir::WalkDir;

use super::explorer::{ContractIdentifier, VerificationInterface};

const CAIRO_EXT: &str = "cairo";
const VERIFY_ENDPOINT: &str = "/class-verify";
const STATUS_ENDPOINT: &str = "/class-verify/job";

pub struct Voyager<'a> {
    network: Network,
    metadata: Metadata,
    provider: &'a JsonRpcClient<HttpTransport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Body {
    pub compiler_version: semver::Version,
    pub scarb_version: semver::Version,
    pub project_dir_path: Utf8PathBuf,
    #[serde(rename = "name")]
    pub contract_name: String,
    pub package_name: String,
    pub license: Option<String>,
    pub files: HashMap<String, String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ApiError {
    error: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct VerificationJobDispatch {
    job_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum VoyagerApiError {
    #[error("Failed to parse {name} path: {path}")]
    DependencyPathError { name: String, path: String },

    #[error("Scarb metadata failed for {name}: {path}")]
    MetadataError { name: String, path: String },
}

fn gather_packages(metadata: &Metadata, packages: &mut Vec<PackageMetadata>) -> Result<()> {
    let mut workspace_packages: Vec<PackageMetadata> = metadata
        .packages
        .clone()
        .into_iter()
        .filter(|package_meta| metadata.workspace.members.contains(&package_meta.id))
        .filter(|package_meta| !packages.contains(package_meta))
        .collect();

    let workspace_packages_names = workspace_packages
        .iter()
        .map(|package| package.name.clone())
        .collect_vec();

    // find all dependencies listed by path
    let mut dependencies: HashMap<String, PathBuf> = HashMap::new();
    for package in &workspace_packages {
        for dependency in &package.dependencies {
            let name = &dependency.name;
            let url = Url::parse(&dependency.source.repr).map_err(|_| {
                VoyagerApiError::DependencyPathError {
                    name: name.clone(),
                    path: dependency.source.repr.clone(),
                }
            })?;

            if url.scheme().starts_with("path") {
                let path =
                    url.to_file_path()
                        .map_err(|()| VoyagerApiError::DependencyPathError {
                            name: name.clone(),
                            path: dependency.source.repr.clone(),
                        })?;
                dependencies.insert(name.clone(), path);
            }
        }
    }

    packages.append(&mut workspace_packages);

    // filter out dependencies already covered by workspace
    let out_of_workspace_dependencies: HashMap<&String, &PathBuf> = dependencies
        .iter()
        .filter(|&(k, _)| !workspace_packages_names.contains(k))
        .collect();

    for (name, manifest) in out_of_workspace_dependencies {
        let new_meta = MetadataCommand::new()
            .json()
            .manifest_path(manifest)
            .exec()
            .map_err(|_| VoyagerApiError::MetadataError {
                name: name.clone(),
                path: manifest.to_string_lossy().to_string(),
            })?;
        gather_packages(&new_meta, packages)?;
    }

    Ok(())
}

fn package_source_files(
    package_metadata: &PackageMetadata,
    include_test_files: bool,
) -> Result<Vec<Utf8PathBuf>> {
    let mut sources: Vec<Utf8PathBuf> = WalkDir::new(package_metadata.root.clone())
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|f| f.file_type().is_file())
        .filter(|f| {
            if let Some(ext) = f.path().extension() {
                if ext != OsStr::new(CAIRO_EXT) {
                    return false;
                }
                let parts: Vec<_> = f
                    .path()
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
                    .collect();

                if parts.contains(&"src".to_string()) {
                    // If include_test_files is true, include all files under src/
                    if include_test_files {
                        return true;
                    }
                    // Otherwise, skip files with "test" in their path under src/
                    let path_str = f.path().to_string_lossy().to_lowercase();
                    if path_str.contains("/test") || path_str.contains("\\test") {
                        return false;
                    }
                    // Also skip files ending with "_test.cairo" or "_tests.cairo"
                    if path_str.ends_with("_test.cairo") || path_str.ends_with("_tests.cairo") {
                        return false;
                    }
                    return true;
                }

                if parts.contains(&"test".to_string()) || parts.contains(&"tests".to_string()) {
                    return false;
                }
                // We'll include files with #[test] attributes since they might be source files
                // that happen to include unit tests
                return true;
            }
            false
        })
        .map(walkdir::DirEntry::into_path)
        .map(Utf8PathBuf::try_from)
        .try_collect()?;

    sources.push(package_metadata.manifest_path.clone());
    let package_root = &package_metadata.root;

    if let Some(lic) = package_metadata
        .manifest_metadata
        .license_file
        .as_ref()
        .map(Utf8Path::new)
        .map(Utf8Path::to_path_buf)
    {
        sources.push(package_root.join(lic));
    }

    if let Some(readme) = package_metadata
        .manifest_metadata
        .readme
        .as_deref()
        .map(Utf8Path::new)
        .map(Utf8Path::to_path_buf)
    {
        sources.push(package_root.join(readme));
    }

    Ok(sources)
}

fn longest_common_prefix<P: AsRef<Utf8Path> + Clone>(
    paths: &[Utf8PathBuf],
    first_guess: P,
) -> Utf8PathBuf {
    let ancestors = Utf8Path::ancestors(first_guess.as_ref());
    let mut longest_prefix = first_guess.as_ref();
    for prefix in ancestors {
        if paths.iter().all(|src| src.starts_with(prefix)) {
            longest_prefix = prefix;
            break;
        }
    }
    longest_prefix.to_path_buf()
}

impl Voyager<'_> {
    pub fn gather_files(
        &self,
        include_test_files: bool,
    ) -> Result<(Utf8PathBuf, HashMap<String, Utf8PathBuf>)> {
        let mut packages = vec![];
        gather_packages(&self.metadata, &mut packages)?;

        let mut sources: Vec<Utf8PathBuf> = vec![];
        for package in &packages {
            let mut package_sources = package_source_files(package, include_test_files)?;
            sources.append(&mut package_sources);
        }

        let prefix = longest_common_prefix(&sources, &self.metadata.workspace.root);
        let manifest_path = &self.metadata.workspace.manifest_path;
        let manifest = manifest_path
            .strip_prefix(&prefix)
            .map_err(|_| anyhow!("Couldn't strip {prefix} from {manifest_path}"))?;

        let mut files: HashMap<String, Utf8PathBuf> = sources
            .iter()
            .map(|p| -> Result<(String, Utf8PathBuf)> {
                let name = p
                    .strip_prefix(&prefix)
                    .map_err(|_| anyhow!("Couldn't strip {prefix} from {p}"))?;
                Ok((name.to_string(), p.clone()))
            })
            .try_collect()?;
        files.insert(
            manifest.to_string(),
            self.metadata.workspace.manifest_path.clone(),
        );

        Ok((prefix, files))
    }
}

#[async_trait::async_trait]
impl<'a> VerificationInterface<'a> for Voyager<'a> {
    fn new(
        network: Network,
        workspace_dir: Utf8PathBuf,
        provider: &'a JsonRpcClient<HttpTransport>,
        _ui: &'a UI,
    ) -> Result<Self> {
        let manifest_path = scarb_utils::get_scarb_manifest_for(workspace_dir.as_ref())?;
        let metadata = scarb_utils::get_scarb_metadata_with_deps(&manifest_path)?;
        Ok(Voyager {
            network,
            metadata,
            provider,
        })
    }

    async fn verify(
        &self,
        contract_identifier: ContractIdentifier,
        contract_name: String,
        package: Option<String>,
        test_files: bool,
        ui: &UI,
    ) -> Result<VerifyResponse> {
        let hash = match contract_identifier {
            ContractIdentifier::ClassHash { class_hash } => Felt::from_hex(class_hash.as_ref())?,
            ContractIdentifier::Address { contract_address } => {
                self.provider
                    .get_class_hash_at(
                        BlockId::Tag(BlockTag::Latest),
                        Felt::from_hex(contract_address.as_ref())?,
                    )
                    .await?
            }
        };

        let cairo_version = self.metadata.app_version_info.cairo.version.clone();
        let scarb_version = self.metadata.app_version_info.version.clone();

        let mut workspace_packages: Vec<PackageMetadata> = self
            .metadata
            .packages
            .iter()
            .filter(|&package_meta| self.metadata.workspace.members.contains(&package_meta.id))
            .cloned()
            .collect();

        let selected = (if workspace_packages.len() > 1 {
            match package {
                Some(ref package_name) => workspace_packages
                    .into_iter()
                    .find(|p| p.name == *package_name)
                    .ok_or(anyhow!(
                        "Package {package_name} not found in scarb metadata"
                    )),
                None => Err(anyhow!(
                    "More than one package found in scarb metadata - specify package using --package flag"
                )),
            }
        } else {
            workspace_packages
                .pop()
                .ok_or(anyhow!("No packages found in scarb metadata"))
        })?;

        let (prefix, files) = self.gather_files(test_files)?;
        let project_dir_path = self
            .metadata
            .workspace
            .root
            .strip_prefix(prefix)
            // backend expects this: "." for cwd
            .map(|path| {
                if path.as_str().is_empty() {
                    Utf8Path::new(".")
                } else {
                    path
                }
            })?;

        if selected.manifest_metadata.license.is_none() {
            ui.println(&WarningMessage::new("License not specified in Scarb.toml"));
        }

        let client = reqwest::Client::new();
        let body = Body {
            compiler_version: cairo_version,
            scarb_version,
            project_dir_path: project_dir_path.to_path_buf(),
            contract_name: contract_name.clone(),
            license: selected.manifest_metadata.license.clone(),
            package_name: selected.name,
            files: files
                .iter()
                .map(|(name, path)| -> Result<(String, String)> {
                    let contents = fs::read_to_string(path.as_path())?;
                    Ok((name.clone(), contents))
                })
                .try_collect()?,
        };

        let url = format!(
            "{}{}/{:#064x}",
            self.gen_explorer_url(),
            VERIFY_ENDPOINT,
            hash
        );

        let response = client
            .post(url)
            .json(&body)
            .send()
            .await
            .context("Failed to submit contract for verification")?;

        match response.status() {
            StatusCode::OK => {
                let message = format!(
                    "{} submitted for verification, you can query the status at: {}{}/{}",
                    contract_name.clone(),
                    self.gen_explorer_url(),
                    STATUS_ENDPOINT,
                    response.json::<VerificationJobDispatch>().await?.job_id,
                );
                Ok(VerifyResponse { message })
            }
            StatusCode::BAD_REQUEST => Err(anyhow!(response.json::<ApiError>().await?.error)),
            _ => Err(anyhow!(response.text().await?)),
        }
    }

    fn gen_explorer_url(&self) -> String {
        match env::var("VERIFIER_API_URL") {
            Ok(addr) => addr,
            Err(_) => match self.network {
                Network::Mainnet => "https://api.voyager.online/beta".to_string(),
                Network::Sepolia => "https://sepolia-api.voyager.online/beta".to_string(),
            },
        }
    }
}
