use crate::ForgeConfigFromScarb;
use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use scarb_metadata::{Metadata, PackageId};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use test_collector::LinkedLibrary;

#[allow(dead_code)]
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct StarknetContract {
    id: String,
    package_name: String,
    contract_name: String,
    artifacts: StarknetContractArtifactPaths,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct StarknetContractArtifactPaths {
    sierra: Utf8PathBuf,
    casm: Option<Utf8PathBuf>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct StarknetArtifacts {
    version: u32,
    contracts: Vec<StarknetContract>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StarknetContractArtifacts {
    pub sierra: String,
    pub casm: Option<String>,
}

/// Get deserialized contents of `starknet_artifacts.json` file generated by Scarb
///
/// # Arguments
///
/// * `path` - A path to `starknet_artifacts.json` file.
pub fn artifacts_for_package(path: &Utf8PathBuf) -> Result<StarknetArtifacts> {
    let starknet_artifacts =
        fs::read_to_string(path).with_context(|| format!("Failed to read {:?} contents", path))?;
    let starknet_artifacts: StarknetArtifacts =
        serde_json::from_str(starknet_artifacts.as_str())
            .with_context(|| format!("Failed to parse {:?} contents. Make sure you have enabled sierra code generation in Scarb.toml", path))?;
    Ok(starknet_artifacts)
}

pub fn try_get_starknet_artifacts_path(path: &Utf8PathBuf) -> Result<Option<Utf8PathBuf>> {
    let path = path.join("target/dev");
    let paths = fs::read_dir(path);
    let Ok(mut paths) = paths else { return Ok(None) };
    let starknet_artifacts = paths.find_map(|path| match path {
        Ok(path) => {
            let name = path.file_name().into_string().ok()?;
            name.contains("starknet_artifacts").then_some(path.path())
        }
        Err(_) => None,
    });
    let starknet_artifacts: Option<Result<Utf8PathBuf>> = starknet_artifacts.map(|artifacts| {
        Utf8PathBuf::try_from(artifacts.clone())
            .with_context(|| format!("Failed to convert path = {artifacts:?} to Utf8PathBuf"))
    });
    starknet_artifacts.transpose()
}

pub fn get_contracts_map(path: &Utf8PathBuf) -> Result<HashMap<String, StarknetContractArtifacts>> {
    let base_path = path
        .parent()
        .ok_or_else(|| anyhow!("Failed to get parent for path = {}", path))?;
    let artifacts = artifacts_for_package(path)?;
    let mut map = HashMap::new();
    for contract in artifacts.contracts {
        let name = contract.contract_name;
        let sierra_path = base_path.join(contract.artifacts.sierra);
        let casm_path = contract
            .artifacts
            .casm
            .map(|casm_path| base_path.join(casm_path));
        let sierra = fs::read_to_string(sierra_path)?;
        let casm: Option<String> = casm_path.map(fs::read_to_string).transpose()?;
        map.insert(name, StarknetContractArtifacts { sierra, casm });
    }
    Ok(map)
}

pub fn config_from_scarb_for_package(
    metadata: &Metadata,
    package: &PackageId,
) -> Result<ForgeConfigFromScarb> {
    let raw_metadata = metadata
        .get_package(package)
        .ok_or_else(|| anyhow!("Failed to find metadata for package = {package}"))?
        .tool_metadata("forge");

    raw_metadata.map_or_else(
        || Ok(Default::default()),
        |raw_metadata| Ok(serde_json::from_value(raw_metadata.clone())?),
    )
}

pub fn dependencies_for_package(
    metadata: &Metadata,
    package: &PackageId,
) -> Result<(Utf8PathBuf, Vec<LinkedLibrary>)> {
    let compilation_unit = metadata
        .compilation_units
        .iter()
        .filter(|unit| unit.package == *package)
        .min_by_key(|unit| match unit.target.name.as_str() {
            name @ "starknet-contract" => (0, name),
            name @ "lib" => (1, name),
            name => (2, name),
        })
        .ok_or_else(|| anyhow!("Failed to find metadata for package = {package}"))?;

    let base_path = metadata
        .get_package(package)
        .ok_or_else(|| anyhow!("Failed to find metadata for package = {package}"))?
        .root
        .clone();

    let dependencies = compilation_unit
        .components
        .iter()
        .filter(|du| !du.source_path.to_string().contains("core/src"))
        .map(|cu| LinkedLibrary {
            name: cu.name.clone(),
            path: cu.source_root().to_owned().into_std_path_buf(),
        })
        .collect();

    Ok((base_path, dependencies))
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::fixture::{FileTouch, FileWriteStr, PathChild, PathCopy};
    use scarb_metadata::MetadataCommand;
    use std::process::Command;

    #[test]
    fn get_starknet_artifacts_path() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from("tests/data/declare_test", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        Command::new("scarb")
            .current_dir(&temp)
            .arg("build")
            .output()
            .unwrap();

        let result = try_get_starknet_artifacts_path(
            &Utf8PathBuf::from_path_buf(temp.to_path_buf()).unwrap(),
        );
        let path = result.unwrap().unwrap();
        assert_eq!(
            path,
            temp.path()
                .join("target/dev/declare_test.starknet_artifacts.json")
        );
    }

    #[test]
    fn get_starknet_artifacts_path_for_project_without_contracts() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_test", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        Command::new("scarb")
            .current_dir(&temp)
            .arg("build")
            .output()
            .unwrap();

        let result = try_get_starknet_artifacts_path(
            &Utf8PathBuf::from_path_buf(temp.to_path_buf()).unwrap(),
        );
        let path = result.unwrap();
        assert!(path.is_none());
    }

    #[test]
    fn get_starknet_artifacts_path_for_project_without_scarb_build() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_test", &["**/*.cairo", "**/*.toml"])
            .unwrap();

        let result = try_get_starknet_artifacts_path(
            &Utf8PathBuf::from_path_buf(temp.to_path_buf()).unwrap(),
        );
        let path = result.unwrap();
        assert!(path.is_none());
    }

    #[test]
    fn parsing_starknet_artifacts() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from("tests/data/dispatchers", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        Command::new("scarb")
            .current_dir(&temp)
            .arg("build")
            .output()
            .unwrap();
        let artifacts_path = temp
            .path()
            .join("target/dev/dispatchers.starknet_artifacts.json");
        let artifacts_path = Utf8PathBuf::from_path_buf(artifacts_path).unwrap();

        let artifacts = artifacts_for_package(&artifacts_path).unwrap();

        assert!(!artifacts.contracts.is_empty());
    }

    #[test]
    fn parsing_starknet_artifacts_on_invalid_file() {
        let temp = assert_fs::TempDir::new().unwrap();
        let path = temp.child("wrong.json");
        path.touch().unwrap();
        path.write_str("\"aa\": {}").unwrap();
        let artifacts_path = Utf8PathBuf::from_path_buf(path.to_path_buf()).unwrap();

        let result = artifacts_for_package(&artifacts_path);
        let err = result.unwrap_err();

        assert!(err.to_string().contains(&format!("Failed to parse {artifacts_path:?} contents. Make sure you have enabled sierra code generation in Scarb.toml")));
    }

    #[test]
    fn get_contracts() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from("tests/data/dispatchers", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        Command::new("scarb")
            .current_dir(&temp)
            .arg("build")
            .output()
            .unwrap();
        let artifacts_path = temp
            .path()
            .join("target/dev/dispatchers.starknet_artifacts.json");
        let artifacts_path = Utf8PathBuf::from_path_buf(artifacts_path).unwrap();

        let contracts = get_contracts_map(&artifacts_path).unwrap();

        assert!(contracts.contains_key("ERC20"));
        assert!(contracts.contains_key("HelloStarknet"));

        let sierra_contents_erc20 =
            fs::read_to_string(temp.join("target/dev/dispatchers_ERC20.sierra.json")).unwrap();
        let casm_contents_erc20 =
            fs::read_to_string(temp.join("target/dev/dispatchers_ERC20.casm.json")).unwrap();
        let contract = contracts.get("ERC20").unwrap();
        assert_eq!(&sierra_contents_erc20, &contract.sierra);
        assert_eq!(&casm_contents_erc20, &contract.casm.clone().unwrap());

        let sierra_contents_erc20 =
            fs::read_to_string(temp.join("target/dev/dispatchers_HelloStarknet.sierra.json"))
                .unwrap();
        let casm_contents_erc20 =
            fs::read_to_string(temp.join("target/dev/dispatchers_HelloStarknet.casm.json"))
                .unwrap();
        let contract = contracts.get("HelloStarknet").unwrap();
        assert_eq!(&sierra_contents_erc20, &contract.sierra);
        assert_eq!(&casm_contents_erc20, &contract.casm.clone().unwrap());
    }

    #[test]
    fn get_dependencies_for_package() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_test", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let (_, dependencies) =
            dependencies_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0])
                .unwrap();

        assert!(!dependencies.is_empty());
        assert!(dependencies.iter().all(|dep| dep.path.exists()));
    }

    #[test]
    fn get_dependencies_for_package_err_on_invalid_package() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_test", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let result =
            dependencies_for_package(&scarb_metadata, &PackageId::from(String::from("12345679")));
        let err = result.unwrap_err();

        assert!(err
            .to_string()
            .contains("Failed to find metadata for package"));
    }

    #[test]
    fn get_forge_config_for_package() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_test", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let config =
            config_from_scarb_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0])
                .unwrap();

        assert_eq!(config, ForgeConfigFromScarb { exit_first: false });
    }

    #[test]
    fn get_forge_config_for_package_err_on_invalid_package() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_test", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let result = config_from_scarb_for_package(
            &scarb_metadata,
            &PackageId::from(String::from("12345679")),
        );
        let err = result.unwrap_err();

        assert!(err
            .to_string()
            .contains("Failed to find metadata for package"));
    }

    #[test]
    fn get_forge_config_for_package_default_on_missing_config() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_test", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let content = "[package]
name = \"example_package\"
version = \"0.1.0\"";
        temp.child("Scarb.toml").write_str(content).unwrap();

        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let config =
            config_from_scarb_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0])
                .unwrap();

        assert_eq!(config, Default::default());
    }
}
