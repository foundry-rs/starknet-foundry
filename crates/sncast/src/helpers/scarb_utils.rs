use anyhow::{anyhow, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use scarb_api::{
    get_contracts_map,
    metadata::{Metadata, MetadataCommand, PackageMetadata},
    ScarbCommand, StarknetContractArtifacts,
};
use std::collections::HashMap;
use std::env;
use std::fs::canonicalize;
use std::str::FromStr;

pub fn get_scarb_manifest() -> Result<Utf8PathBuf> {
    get_scarb_manifest_for(<&Utf8Path>::from("."))
}

pub fn get_scarb_manifest_for(dir: &Utf8Path) -> Result<Utf8PathBuf> {
    ScarbCommand::new().ensure_available()?;

    let output = ScarbCommand::new()
        .current_dir(dir)
        .arg("manifest-path")
        .command()
        .output()
        .context("Failed to execute the `scarb manifest-path` command")?;

    let output_str = String::from_utf8(output.stdout)
        .context("`scarb manifest-path` command failed to provide valid output")?;

    let path = Utf8PathBuf::from_str(output_str.trim())
        .context("`scarb manifest-path` failed. Invalid location returned")?;

    Ok(path)
}

fn get_scarb_metadata_command(manifest_path: &Utf8PathBuf) -> Result<MetadataCommand> {
    ScarbCommand::new().ensure_available()?;

    let mut command = ScarbCommand::metadata();
    command.inherit_stderr().manifest_path(manifest_path);
    Ok(command)
}

fn execute_scarb_metadata_command(command: &MetadataCommand) -> Result<Metadata> {
    command.exec().context(format!(
        "Failed to read the `Scarb.toml` manifest file. Doesn't exist in the current or parent directories = {}",
        env::current_dir()
            .expect("Failed to access the current directory")
            .into_os_string()
            .into_string()
            .expect("Failed to convert current directory into a string")
    ))
}

pub fn get_scarb_metadata(manifest_path: &Utf8PathBuf) -> Result<Metadata> {
    let mut command = get_scarb_metadata_command(manifest_path)?;
    let command = command.no_deps();
    execute_scarb_metadata_command(command)
}

pub fn get_scarb_metadata_with_deps(manifest_path: &Utf8PathBuf) -> Result<Metadata> {
    let command = get_scarb_metadata_command(manifest_path)?;
    execute_scarb_metadata_command(&command)
}

#[must_use]
pub fn verify_or_determine_scarb_manifest_path(
    path_to_scarb_toml: &Option<Utf8PathBuf>,
) -> Option<Utf8PathBuf> {
    if let Some(path) = path_to_scarb_toml {
        assert!(path.exists(), "Failed to locate file at path = {path}");
    }

    let manifest_path = path_to_scarb_toml.clone().unwrap_or_else(|| {
        get_scarb_manifest()
            .context("Failed to obtain manifest path from scarb")
            .unwrap()
    });

    if !manifest_path.exists() {
        return None;
    }

    Some(manifest_path)
}

pub fn get_package_metadata<'a>(
    metadata: &'a Metadata,
    manifest_path: &'a Utf8PathBuf,
) -> Result<&'a PackageMetadata> {
    let manifest_path = canonicalize(manifest_path.clone())
        .unwrap_or_else(|err| panic!("Failed to canonicalize {manifest_path}, error: {err:?}"));

    let package = metadata
        .packages
        .iter()
        .find(|package| package.manifest_path == manifest_path)
        .ok_or(anyhow!(
            "Path = {} not found in scarb metadata",
            manifest_path.display()
        ))?;
    Ok(package)
}

pub struct BuildConfig {
    pub scarb_toml_path: Utf8PathBuf,
    pub json: bool,
}

pub fn build(config: &BuildConfig) -> Result<HashMap<String, StarknetContractArtifacts>> {
    let mut cmd = ScarbCommand::new_with_stdio();
    cmd.arg("build").manifest_path(&config.scarb_toml_path);
    if config.json {
        cmd.json();
    }
    cmd.run()
        .map_err(|e| anyhow!(format!("Failed to build using scarb; {e}")))?;

    let metadata = get_scarb_metadata_command(&config.scarb_toml_path)?
        .exec()
        .expect("Failed to obtain metadata");
    let package = get_package_metadata(&metadata, &config.scarb_toml_path)
        .with_context(|| anyhow!("Failed to find package"))?;
    get_contracts_map(&metadata, &package.id)
}

#[cfg(test)]
mod tests {
    use crate::helpers::scarb_utils::get_scarb_metadata;

    #[test]
    fn test_get_scarb_metadata() {
        let metadata = get_scarb_metadata(&"tests/data/contracts/map/Scarb.toml".into());
        assert!(metadata.is_ok());
    }

    #[test]
    fn test_get_scarb_metadata_not_found() {
        let metadata_err = get_scarb_metadata(&"Scarb.toml".into()).unwrap_err();
        assert!(metadata_err
            .to_string()
            .contains("Failed to read the `Scarb.toml` manifest file."));
    }
}
