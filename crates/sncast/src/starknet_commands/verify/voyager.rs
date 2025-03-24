use anyhow::{Context, Result, anyhow};
use camino::{Utf8Path, Utf8PathBuf};
use itertools::Itertools;
use reqwest::{self, StatusCode};
use scarb_api::metadata::MetadataCommand;
use scarb_metadata::{Metadata, PackageMetadata};
use serde::Serialize;
use shared::print::print_as_warning;
use sncast::Network;
use sncast::{helpers::scarb_utils, response::structs::VerifyResponse};
use starknet::{
    core::types::{BlockId, BlockTag},
    providers::{
        Provider,
        jsonrpc::{HttpTransport, JsonRpcClient},
    },
};
use starknet_types_core::felt::Felt;
use std::{collections::HashMap, env, ffi::OsStr, fmt, fs, path::PathBuf};
use url::Url;
use walkdir::WalkDir;

use super::explorer::{ContractIdentifier, VerificationInterface};

const CAIRO_EXT: &str = "cairo";

pub struct Voyager {
    network: Network,
    metadata: Metadata,
    provider: JsonRpcClient<HttpTransport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Body {
    pub compiler_version: semver::Version,
    pub scarb_version: semver::Version,
    pub project_dir_path: Utf8PathBuf,
    pub name: String,
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

fn dependency_path<S>(name: S, path: S) -> anyhow::Error
where
    S: fmt::Display,
{
    anyhow!("Couldn't parse {name} path: {path}")
}

fn metadata_error<S>(name: S, path: S) -> anyhow::Error
where
    S: fmt::Display,
{
    anyhow!("Scarb metadata failed for {name}: {path}")
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
            let url = Url::parse(&dependency.source.repr)
                .map_err(|_| dependency_path(name.clone(), dependency.source.repr.clone()))?;

            if url.scheme().starts_with("path") {
                let path = url
                    .to_file_path()
                    .map_err(|()| dependency_path(name.clone(), dependency.source.repr.clone()))?;
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
            .map_err(|_| metadata_error(name.clone(), manifest.to_string_lossy().to_string()))?;
        gather_packages(&new_meta, packages)?;
    }

    Ok(())
}

fn package_sources(package_metadata: &PackageMetadata) -> Result<Vec<Utf8PathBuf>> {
    let mut sources: Vec<Utf8PathBuf> = WalkDir::new(package_metadata.root.clone())
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|f| f.file_type().is_file())
        .filter(|f| {
            if let Some(ext) = f.path().extension() {
                if ext == OsStr::new(CAIRO_EXT) {
                    return true;
                }
            };

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

fn biggest_common_prefix<P: AsRef<Utf8Path> + Clone>(
    paths: &[Utf8PathBuf],
    first_guess: P,
) -> Utf8PathBuf {
    let ancestors = Utf8Path::ancestors(first_guess.as_ref());
    let mut biggest_prefix: &Utf8Path = first_guess.as_ref();
    for prefix in ancestors {
        if paths.iter().all(|src| src.starts_with(prefix)) {
            biggest_prefix = prefix;
            break;
        }
    }
    biggest_prefix.to_path_buf()
}

impl Voyager {
    fn gather_files(&self) -> Result<(Utf8PathBuf, HashMap<String, Utf8PathBuf>)> {
        let mut packages = vec![];
        gather_packages(&self.metadata, &mut packages)?;

        let mut sources: Vec<Utf8PathBuf> = vec![];
        for package in &packages {
            let mut package_sources = package_sources(package)?;
            sources.append(&mut package_sources);
        }

        let prefix = biggest_common_prefix(&sources, &self.metadata.workspace.root);
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
impl VerificationInterface for Voyager {
    fn new(network: Network, workspace_dir: Utf8PathBuf) -> Result<Self> {
        let manifest_path = scarb_utils::get_scarb_manifest_for(workspace_dir.as_ref())?;
        let metadata = scarb_utils::get_scarb_metadata_with_deps(&manifest_path)?;
        let url_str = match env::var("STARKNET_RPC_URL") {
            Ok(addr) => addr,
            Err(_) => match network {
                Network::Mainnet => "https://api.voyager.online/beta".to_string(),
                Network::Sepolia => "https://sepolia-api.voyager.online/beta".to_string(),
            },
        };
        let url = Url::parse(&url_str)?;
        let provider = JsonRpcClient::new(HttpTransport::new(url));
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
                Some(ref package_name) => {
                    if let Some(p) = workspace_packages
                        .into_iter()
                        .find(|p| p.name == *package_name)
                    {
                        Ok(p)
                    } else {
                        Err(anyhow!(
                            "Package {} not found in scarb metadata",
                            package_name
                        ))
                    }
                }
                None => Err(anyhow!(
                    "More than one package found in scarb metadata - specify package using --package flag"
                )),
            }
        } else if let Some(p) = workspace_packages.pop() {
            Ok(p)
        } else {
            Err(anyhow!("No packages found in scarb metadata"))
        })?;

        let (prefix, files) = self.gather_files()?;
        let project_dir_path = self
            .metadata
            .workspace
            .root
            .strip_prefix(prefix)
            // backend expects this: "." for cwd
            .map(|p| {
                if p.as_str().is_empty() {
                    Utf8Path::new(".")
                } else {
                    p
                }
            })?;

        println!("The following files will be transferred:");
        for (name, path) in &files {
            println!("{name}: \n{path}");
        }

        if selected.manifest_metadata.license.is_none() {
            print_as_warning(&anyhow!("License not specified in Scarb.toml"));
        }

        let client = reqwest::Client::new();
        let body = Body {
            compiler_version: cairo_version,
            scarb_version,
            project_dir_path: project_dir_path.to_path_buf(),
            name: contract_name.clone(),
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

        let url = format!("{}/class-verify/{:#068x}", self.gen_explorer_url(), hash);

        let response = client
            .post(url)
            .json(&body)
            .send()
            .await
            .context("Failed to submit contract for verification")?;

        match response.status() {
            StatusCode::OK => {
                let message = format!(
                    "{} submitted for verification, you can query the status at: {}/class-verify/job/{}",
                    contract_name.clone(),
                    self.gen_explorer_url(),
                    response.json::<VerificationJobDispatch>().await?.job_id,
                );
                Ok(VerifyResponse { message })
            }
            StatusCode::BAD_REQUEST => Err(anyhow!(response.json::<ApiError>().await?.error)),
            _ => Err(anyhow!(response.text().await?)),
        }
    }

    fn gen_explorer_url(&self) -> String {
        match env::var("VOYAGER_API_URL") {
            Ok(addr) => addr,
            Err(_) => match self.network {
                Network::Mainnet => "https://api.voyager.online/beta".to_string(),
                Network::Sepolia => "https://sepolia-api.voyager.online/beta".to_string(),
            },
        }
    }
}
