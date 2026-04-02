use super::explorer::{ContractIdentifier, VerificationInterface};
use anyhow::{Context, Result, anyhow};
use camino::{Utf8Path, Utf8PathBuf};
use foundry_ui::components::warning::WarningMessage;
use itertools::Itertools;
use reqwest::{self, StatusCode};
use scarb_api::metadata::metadata_for_dir;
use scarb_metadata::{Metadata, PackageMetadata};
use serde::Serialize;
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
use std::{collections::HashMap, env, ffi::OsStr, fs, path::PathBuf};
use url::Url;
use walkdir::WalkDir;

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
    pub name: String,
    pub contract_file: String,
    #[serde(rename = "contract-name")]
    pub serialized_contract_name: String,
    pub package_name: String,
    pub build_tool: String,
    pub license: String,
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

fn select_package(metadata: &Metadata, package: Option<&str>) -> Result<PackageMetadata> {
    let mut workspace_packages: Vec<PackageMetadata> = metadata
        .packages
        .iter()
        .filter(|&package_meta| metadata.workspace.members.contains(&package_meta.id))
        .cloned()
        .collect();

    if workspace_packages.len() > 1 {
        match package {
            Some(package_name) => workspace_packages
                .into_iter()
                .find(|p| p.name == package_name)
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
    }
}

fn prepare_project_dir_path() -> Utf8PathBuf {
    Utf8PathBuf::from(".")
}

fn build_files_payload(files: &HashMap<String, Utf8PathBuf>) -> Result<HashMap<String, String>> {
    files
        .iter()
        .map(|(name, path)| -> Result<(String, String)> {
            let mut contents = fs::read_to_string(path.as_path())?;
            if is_scarb_manifest(name) {
                contents = filter_scarb_toml_content(&contents);
            }
            Ok((name.clone(), contents))
        })
        .try_collect()
}

fn build_request_body(
    cairo_version: semver::Version,
    scarb_version: semver::Version,
    contract_name: &str,
    selected: &PackageMetadata,
    prefix: &Utf8Path,
    files: &HashMap<String, Utf8PathBuf>,
) -> Result<Body> {
    let contract_file = find_contract_file(files, &selected.root, contract_name, prefix)?;

    Ok(Body {
        compiler_version: cairo_version,
        scarb_version,
        project_dir_path: prepare_project_dir_path(),
        name: contract_name.to_string(),
        contract_file: contract_file.clone(),
        serialized_contract_name: serialized_contract_name(&contract_file),
        package_name: selected.name.clone(),
        build_tool: "scarb".to_string(),
        license: normalize_license(selected.manifest_metadata.license.as_deref()),
        files: build_files_payload(files)?,
    })
}

fn serialized_contract_name(contract_file: &str) -> String {
    contract_file.to_string()
}

fn normalize_license(license: Option<&str>) -> String {
    license.unwrap_or("NONE").to_string()
}

fn is_scarb_manifest(path: &str) -> bool {
    path == "Scarb.toml" || path.ends_with("/Scarb.toml")
}

fn filter_scarb_toml_content(content: &str) -> String {
    let mut lines = Vec::new();
    let mut in_dev_dependencies = false;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("[dev-dependencies]") {
            in_dev_dependencies = true;
            lines.push("# [dev-dependencies] section removed for remote compilation");
            continue;
        }

        if trimmed.starts_with('[') && !trimmed.starts_with("[dev-dependencies]") {
            if in_dev_dependencies {
                lines.push("");
            }
            in_dev_dependencies = false;
            lines.push(line);
            continue;
        }

        if in_dev_dependencies {
            continue;
        }

        lines.push(line);
    }

    lines.join("\n")
}

fn find_contract_file_path(
    files: &HashMap<String, Utf8PathBuf>,
    package_root: &Utf8Path,
    contract_name: &str,
) -> Result<Utf8PathBuf> {
    if let Some(contract_file) = find_contract_by_pattern(files, package_root, contract_name) {
        return Ok(contract_file);
    }

    for path in [
        format!("src/{}.cairo", contract_name.to_lowercase()),
        format!(
            "src/{}/{}.cairo",
            contract_name.to_lowercase(),
            contract_name.to_lowercase()
        ),
        format!("src/systems/{}.cairo", contract_name.to_lowercase()),
        format!("src/contracts/{}.cairo", contract_name.to_lowercase()),
    ] {
        let full_path = package_root.join(path);
        if full_path.exists() {
            return Ok(full_path);
        }
    }

    for path in ["src/lib.cairo", "src/main.cairo"] {
        let full_path = package_root.join(path);
        if full_path.exists() {
            return Ok(full_path);
        }
    }

    files
        .values()
        .filter(|path| path.starts_with(package_root))
        .find(|path| path.extension() == Some(CAIRO_EXT))
        .cloned()
        .ok_or(anyhow!(
            "No Cairo source files found for package {package_root}"
        ))
}

fn find_contract_file(
    files: &HashMap<String, Utf8PathBuf>,
    package_root: &Utf8Path,
    contract_name: &str,
    prefix: &Utf8Path,
) -> Result<String> {
    let contract_file_path = find_contract_file_path(files, package_root, contract_name)?;
    let contract_file = contract_file_path
        .strip_prefix(prefix)
        .map_err(|_| anyhow!("Couldn't strip {prefix} from {contract_file_path}"))?;
    Ok(contract_file.to_string())
}

fn find_contract_by_pattern(
    files: &HashMap<String, Utf8PathBuf>,
    package_root: &Utf8Path,
    contract_name: &str,
) -> Option<Utf8PathBuf> {
    files
        .values()
        .filter(|path| path.starts_with(package_root))
        .filter(|path| path.extension() == Some(CAIRO_EXT))
        .find_map(|path| match fs::read_to_string(path) {
            Ok(contents) if contains_contract_definition(&contents, contract_name) => {
                Some(path.clone())
            }
            _ => None,
        })
}

fn contains_contract_definition(content: &str, contract_name: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();

    for (index, line) in lines.iter().enumerate() {
        if !line.trim().starts_with("#[starknet::contract]") {
            continue;
        }

        let end_index = std::cmp::min(index + 5, lines.len());
        for next_line in lines.iter().skip(index + 1).take(end_index - (index + 1)) {
            let next_line = next_line.trim();
            if next_line.is_empty() || next_line.starts_with("//") {
                continue;
            }

            if let Some(module_name) = extract_module_name(next_line) {
                if module_name == contract_name {
                    return true;
                }
                break;
            }
        }
    }

    false
}

fn extract_module_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let without_pub = trimmed
        .strip_prefix("pub ")
        .map_or(trimmed, |rest| rest.trim());

    without_pub
        .strip_prefix("mod ")
        .and_then(|rest| {
            rest.trim()
                .split(|c: char| c == '{' || c.is_whitespace())
                .next()
        })
        .filter(|name| !name.is_empty())
        .map(ToString::to_string)
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
        let selected = select_package(&self.metadata, package.as_deref())?;

        let (prefix, files) = self.gather_files(test_files)?;

        if selected.manifest_metadata.license.is_none() {
            ui.print_warning(WarningMessage::new("License not specified in Scarb.toml"));
        }

        let client = reqwest::Client::new();
        let body = build_request_body(
            cairo_version,
            scarb_version,
            &contract_name,
            &selected,
            &prefix,
            &files,
        )?;

        let url = format!(
            "{}{}/{:#066x}",
            self.gen_explorer_url()?,
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
                    self.gen_explorer_url()?,
                    STATUS_ENDPOINT,
                    response.json::<VerificationJobDispatch>().await?.job_id,
                );
                Ok(VerifyResponse { message })
            }
            StatusCode::BAD_REQUEST => Err(anyhow!(response.json::<ApiError>().await?.error)),
            _ => Err(anyhow!(response.text().await?)),
        }
    }

    fn gen_explorer_url(&self) -> Result<String> {
        match env::var("VERIFIER_API_URL") {
            Ok(addr) => Ok(addr),
            Err(_) => match self.network {
                Network::Mainnet => Ok("https://api.voyager.online/beta".to_string()),
                Network::Sepolia => Ok("https://sepolia-api.voyager.online/beta".to_string()),
                Network::Devnet => Err(ExplorerError::DevnetNotSupported.into()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        contains_contract_definition, filter_scarb_toml_content, find_contract_file,
        find_contract_file_path, is_scarb_manifest, normalize_license, prepare_project_dir_path,
        serialized_contract_name,
    };
    use camino::{Utf8Path, Utf8PathBuf};
    use std::collections::HashMap;
    use walkdir::WalkDir;

    fn contract_fixture_path(path: &str) -> Utf8PathBuf {
        Utf8Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/contracts")
            .join(path)
    }

    fn fixture_files(root: &Utf8Path) -> HashMap<String, Utf8PathBuf> {
        WalkDir::new(root)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| Utf8PathBuf::try_from(entry.into_path()).expect("utf-8 test path"))
            .map(|path| (path.as_str().to_string(), path))
            .collect()
    }

    #[test]
    fn test_contract_file_detection_matches_contract_module() {
        let contents = r"
#[starknet::contract]
pub mod MatchingContract {
}

#[starknet::contract]
mod OtherContract {
}
";

        assert!(contains_contract_definition(contents, "MatchingContract"));
        assert!(contains_contract_definition(contents, "OtherContract"));
        assert!(!contains_contract_definition(contents, "MissingContract"));
    }

    #[test]
    fn test_contract_file_map_fixture_resolves_to_src_lib_cairo() {
        let package_root = contract_fixture_path("map");
        let files = fixture_files(&package_root);

        let contract_file_path =
            find_contract_file_path(&files, &package_root, "Map").expect("contract file");
        let contract_file =
            find_contract_file(&files, &package_root, "Map", &package_root).expect("relative path");

        assert_eq!(contract_file_path, package_root.join("src/lib.cairo"));
        assert_eq!(contract_file, "src/lib.cairo");
    }

    #[test]
    fn test_contract_file_virtual_workspace_resolves_to_workspace_relative_path() {
        let workspace_root = contract_fixture_path("virtual_workspace");
        let package_root = workspace_root.join("crates/cast_fibonacci");
        let files = fixture_files(&workspace_root);

        let contract_file =
            find_contract_file(&files, &package_root, "FibonacciContract", &workspace_root)
                .expect("relative contract file");

        assert_eq!(contract_file, "crates/cast_fibonacci/src/lib.cairo");
    }

    #[test]
    fn test_project_dir_path_is_dot() {
        assert_eq!(prepare_project_dir_path(), Utf8PathBuf::from("."));
    }

    #[test]
    fn test_serialized_contract_name_mirrors_contract_file() {
        assert_eq!(
            serialized_contract_name("src/lib.cairo"),
            "src/lib.cairo".to_string()
        );
    }

    #[test]
    fn test_normalize_license_defaults_to_none() {
        assert_eq!(normalize_license(None), "NONE".to_string());
        assert_eq!(normalize_license(Some("MIT")), "MIT".to_string());
    }

    #[test]
    fn test_filter_scarb_toml_content_removes_dev_dependencies_only() {
        let manifest = r#"[package]
name = "map"

[dependencies]
starknet = "2.0.0"

[dev-dependencies]
snforge_std = "0.43.0"

[scripts]
test = "snforge test"
"#;

        let filtered = filter_scarb_toml_content(manifest);

        assert!(!filtered.contains("snforge_std"));
        assert!(filtered.contains("[dependencies]"));
        assert!(filtered.contains("[scripts]"));
    }

    #[test]
    fn test_is_scarb_manifest_matches_only_manifest_paths() {
        assert!(is_scarb_manifest("Scarb.toml"));
        assert!(is_scarb_manifest("crates/cast_fibonacci/Scarb.toml"));
        assert!(!is_scarb_manifest("Scarb.lock"));
    }
}
