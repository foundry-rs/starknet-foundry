use super::explorer::{ContractIdentifier, VerificationInterface};
use anyhow::Result;
use camino::Utf8PathBuf;
use foundry_ui::components::warning::WarningMessage;
use scarb_metadata::Metadata;
use sncast::Network;
use sncast::response::explorer_link::ExplorerError;
use sncast::response::ui::UI;
use sncast::{helpers::scarb_utils, response::verify::VerifyResponse};
use starknet_rust::{
    core::types::{BlockId, BlockTag},
    providers::{
        Provider,
        jsonrpc::{HttpTransport, JsonRpcClient},
    },
};
use starknet_types_core::felt::Felt;
use std::{collections::HashMap, env};
use url::Url;
use voyager_verifier::{
    core::project::ProjectType,
    voyager::{
        MAINNET_API_URL, SEPOLIA_API_URL, STATUS_ENDPOINT, collect_verification_files,
        prepare_verification_request, submit_verification_request,
    },
};

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
    pub build_tool: String,
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
        let new_meta = metadata_for_dir(manifest.parent().expect("manifest should have a parent"))
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
                    if path_str.contains("/test") {
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
        let files = collect_verification_files(&self.metadata, include_test_files)?;

        Ok((files.prefix, files.files))
    }

    fn get_job_status_url(base_url: &Url, job_id: &str) -> Result<Url> {
        let mut url = base_url.clone();
        let url_clone = url.clone();
        url.path_segments_mut()
            .map_err(|()| anyhow::anyhow!("Voyager API URL cannot be used as a base: {url_clone}"))?
            .extend(STATUS_ENDPOINT.split('/').chain(std::iter::once(job_id)));
        Ok(url)
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
        let class_hash = match contract_identifier {
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

        let prepared = prepare_verification_request(
            &self.metadata,
            &contract_name,
            package.as_deref(),
            test_files,
            ProjectType::Scarb,
            None,
        )?;

        if prepared.package.manifest_metadata.license.is_none() {
            ui.print_warning(WarningMessage::new("License not specified in Scarb.toml"));
        }

        let explorer_url = self.gen_explorer_url()?;
        let api_base_url = Url::parse(&explorer_url)?;
        let class_hash = format!("{class_hash:#066x}");
        let job_id =
            submit_verification_request(&api_base_url, &class_hash, &prepared.request).await?;
        let status_url = Self::get_job_status_url(&api_base_url, &job_id)?;
        let message = format!(
            "{contract_name} submitted for verification, you can query the status at: {status_url}"
        );

        Ok(VerifyResponse { message })
    }

    fn gen_explorer_url(&self) -> Result<String> {
        match env::var("VERIFIER_API_URL") {
            Ok(addr) => Ok(addr),
            Err(_) => match self.network {
                Network::Mainnet => Ok(MAINNET_API_URL.to_string()),
                Network::Sepolia => Ok(SEPOLIA_API_URL.to_string()),
                Network::Devnet => Err(ExplorerError::DevnetNotSupported.into()),
            },
        }
    }
}
