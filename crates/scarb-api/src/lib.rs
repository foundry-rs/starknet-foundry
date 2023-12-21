use anyhow::{anyhow, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use scarb_metadata::{CompilationUnitMetadata, Metadata, PackageId};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

pub use command::*;
use sierra_casm::compile;

mod command;

#[derive(Deserialize, Debug, PartialEq, Clone)]
struct StarknetArtifacts {
    version: u32,
    contracts: Vec<StarknetContract>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, PartialEq, Clone)]
struct StarknetContract {
    id: String,
    package_name: String,
    contract_name: String,
    artifacts: StarknetContractArtifactPaths,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, PartialEq, Clone)]
struct StarknetContractArtifactPaths {
    sierra: Utf8PathBuf,
    casm: Option<Utf8PathBuf>,
}

/// Contains compiled Starknet artifacts
#[derive(Debug, PartialEq, Clone)]
pub struct StarknetContractArtifacts {
    /// Compiled sierra code
    pub sierra: String,
    /// Compiled casm code
    pub casm: String,
}

impl StarknetContractArtifacts {
    fn from_scarb_contract_artifact(
        starknet_contract: &StarknetContract,
        base_path: &Utf8Path,
    ) -> Result<Self> {
        let sierra_path = base_path.join(starknet_contract.artifacts.sierra.clone());
        let sierra = fs::read_to_string(sierra_path)?;

        let casm = match &starknet_contract.artifacts.casm {
            None => String::new(),
            Some(casm_path) => fs::read_to_string(base_path.join(casm_path))?,
        };

        Ok(Self { sierra, casm })
    }
}

/// Get deserialized contents of `starknet_artifacts.json` file generated by Scarb
///
/// # Arguments
///
/// * `path` - A path to `starknet_artifacts.json` file.
fn artifacts_for_package(path: &Utf8Path) -> Result<StarknetArtifacts> {
    let starknet_artifacts =
        fs::read_to_string(path).with_context(|| format!("Failed to read {path:?} contents"))?;
    let starknet_artifacts: StarknetArtifacts =
        serde_json::from_str(starknet_artifacts.as_str())
            .with_context(|| format!("Failed to parse {path:?} contents. Make sure you have enabled sierra code generation in Scarb.toml"))?;
    Ok(starknet_artifacts)
}

/// Try getting path to Scarb starknet artifacts for the given package
///
/// Try getting the path to `starknet_artifacts.json` file that is generated by `scarb build` command.
/// If the file is not present, for example when package doesn't define the starknet target, `None` is returned.
///
/// # Arguments
///
/// * `target_dir` - A path to the target directory of the package
/// * `target_name` - A name of the target that is being built by Scarb
fn try_get_starknet_artifacts_path(
    target_dir: &Utf8Path,
    target_name: &str,
    current_profile: &str,
) -> Result<Option<Utf8PathBuf>> {
    let path = target_dir.join(current_profile);
    let paths = fs::read_dir(path);
    let Ok(mut paths) = paths else {
        return Ok(None);
    };
    let starknet_artifacts = paths.find_map(|path| match path {
        Ok(path) => {
            let name = path.file_name().into_string().ok()?;
            (name == format!("{target_name}.starknet_artifacts.json")).then_some(path.path())
        }
        Err(_) => None,
    });
    let starknet_artifacts: Option<Result<Utf8PathBuf>> = starknet_artifacts.map(|artifacts| {
        Utf8PathBuf::try_from(artifacts.clone())
            .with_context(|| format!("Failed to convert path = {artifacts:?} to Utf8PathBuf"))
    });
    starknet_artifacts.transpose()
}

/// Get the map with `StarknetContractArtifacts` for the given package
pub fn get_contracts_map(
    metadata: &Metadata,
    package: &PackageId,
) -> Result<HashMap<String, StarknetContractArtifacts>> {
    let target_name = target_name_for_package(metadata, package)?;
    let target_dir = target_dir_for_workspace(metadata);
    let maybe_contracts_path =
        try_get_starknet_artifacts_path(&target_dir, &target_name, &metadata.current_profile)?;

    let map = match maybe_contracts_path {
        Some(contracts_path) => {
            let mut contracts = load_contract_artifacts(&contracts_path)?;

            for (_, artifact) in &mut contracts.iter_mut() {
                if artifact.casm.is_empty() {
                    let sierra: Value = serde_json::from_str(&artifact.sierra)?;
                    artifact.casm = serde_json::to_string(&compile(sierra)?)?;
                }
            }

            contracts
        }
        None => HashMap::default(),
    };
    Ok(map)
}

fn load_contract_artifacts(
    contracts_path: &Utf8PathBuf,
) -> Result<HashMap<String, StarknetContractArtifacts>> {
    let base_path = contracts_path
        .parent()
        .ok_or_else(|| anyhow!("Failed to get parent for path = {}", &contracts_path))?;
    let artifacts = artifacts_for_package(contracts_path)?;
    let mut map = HashMap::new();

    for ref contract in artifacts.contracts {
        let name = contract.contract_name.clone();
        let contract_artifacts =
            StarknetContractArtifacts::from_scarb_contract_artifact(contract, base_path)?;
        map.insert(name, contract_artifacts);
    }
    Ok(map)
}

fn compilation_unit_for_package<'a>(
    metadata: &'a Metadata,
    package: &PackageId,
) -> Result<&'a CompilationUnitMetadata> {
    metadata
        .compilation_units
        .iter()
        .filter(|unit| unit.package == *package)
        .min_by_key(|unit| match unit.target.kind.as_str() {
            name @ "starknet-contract" => (0, name),
            name @ "lib" => (1, name),
            name => (2, name),
        })
        .ok_or_else(|| anyhow!("Failed to find metadata for package = {package}"))
}

/// Get the target name for the given package
pub fn target_name_for_package(metadata: &Metadata, package: &PackageId) -> Result<String> {
    let compilation_unit = compilation_unit_for_package(metadata, package)?;
    Ok(compilation_unit.target.name.clone())
}

#[must_use]
pub fn target_dir_for_workspace(metadata: &Metadata) -> Utf8PathBuf {
    metadata
        .target_dir
        .clone()
        .unwrap_or_else(|| metadata.workspace.root.join("target"))
}

/// Get a name of the given package
pub fn name_for_package(metadata: &Metadata, package: &PackageId) -> Result<String> {
    let package = metadata
        .get_package(package)
        .ok_or_else(|| anyhow!("Failed to find metadata for package = {package}"))?;

    Ok(package.name.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
    use assert_fs::prelude::FileTouch;
    use assert_fs::TempDir;
    use camino::Utf8PathBuf;
    use indoc::{formatdoc, indoc};
    use scarb_metadata::MetadataCommand;
    use std::str::FromStr;

    fn setup_package(package_name: &str) -> TempDir {
        let temp = TempDir::new().unwrap();
        temp.copy_from(
            format!("tests/data/{package_name}"),
            &["**/*.cairo", "**/*.toml"],
        )
        .unwrap();
        temp.copy_from("../../", &[".tool-versions"]).unwrap();

        let snforge_std_path = Utf8PathBuf::from_str("../../snforge_std")
            .unwrap()
            .canonicalize_utf8()
            .unwrap()
            .to_string()
            .replace('\\', "/");

        let manifest_path = temp.child("Scarb.toml");
        manifest_path
            .write_str(&formatdoc!(
                r#"
                [package]
                name = "{}"
                version = "0.1.0"

                [[target.starknet-contract]]
                sierra = true
                casm = true

                [dependencies]
                starknet = "2.4.0"
                snforge_std = {{ path = "{}" }}

                [[tool.snforge.fork]]
                name = "FIRST_FORK_NAME"
                url = "http://some.rpc.url"
                block_id.number = "1"

                [[tool.snforge.fork]]
                name = "SECOND_FORK_NAME"
                url = "http://some.rpc.url"
                block_id.hash = "1"

                [[tool.snforge.fork]]
                name = "THIRD_FORK_NAME"
                url = "http://some.rpc.url"
                block_id.tag = "Latest"
                "#,
                package_name,
                snforge_std_path
            ))
            .unwrap();

        temp
    }

    #[test]
    fn get_starknet_artifacts_path() {
        let temp = setup_package("basic_package");

        ScarbCommand::new_with_stdio()
            .current_dir(temp.path())
            .arg("build")
            .run()
            .unwrap();

        let result = try_get_starknet_artifacts_path(
            &Utf8PathBuf::from_path_buf(temp.to_path_buf().join("target")).unwrap(),
            "basic_package",
            "dev",
        );
        let path = result.unwrap().unwrap();
        assert_eq!(
            path,
            temp.path()
                .join("target/dev/basic_package.starknet_artifacts.json")
        );
    }

    #[test]
    fn get_starknet_artifacts_path_for_project_with_different_package_and_target_name() {
        let temp = setup_package("basic_package");

        let snforge_std_path = Utf8PathBuf::from_str("../../snforge_std")
            .unwrap()
            .canonicalize_utf8()
            .unwrap()
            .to_string()
            .replace('\\', "/");

        let scarb_path = temp.child("Scarb.toml");
        scarb_path
            .write_str(&formatdoc!(
                r#"
                [package]
                name = "basic_package"
                version = "0.1.0"

                [dependencies]
                starknet = "2.4.0"
                snforge_std = {{ path = "{}" }}

                [[target.starknet-contract]]
                name = "essa"
                sierra = true
                casm = true
                "#,
                snforge_std_path
            ))
            .unwrap();

        ScarbCommand::new_with_stdio()
            .current_dir(temp.path())
            .arg("build")
            .run()
            .unwrap();

        let result = try_get_starknet_artifacts_path(
            &Utf8PathBuf::from_path_buf(temp.to_path_buf().join("target")).unwrap(),
            "essa",
            "dev",
        );
        let path = result.unwrap().unwrap();
        assert_eq!(
            path,
            temp.path().join("target/dev/essa.starknet_artifacts.json")
        );
    }

    #[test]
    fn get_starknet_artifacts_path_for_project_without_starknet_target() {
        let temp = setup_package("empty_lib");

        let manifest_path = temp.child("Scarb.toml");
        manifest_path
            .write_str(indoc!(
                r#"
            [package]
            name = "empty_lib"
            version = "0.1.0"
            "#,
            ))
            .unwrap();

        ScarbCommand::new_with_stdio()
            .current_dir(temp.path())
            .arg("build")
            .run()
            .unwrap();

        let result = try_get_starknet_artifacts_path(
            &Utf8PathBuf::from_path_buf(temp.to_path_buf().join("target")).unwrap(),
            "empty_lib",
            "dev",
        );
        let path = result.unwrap();
        assert!(path.is_none());
    }

    #[test]
    fn get_starknet_artifacts_path_for_project_without_scarb_build() {
        let temp = setup_package("basic_package");

        let result = try_get_starknet_artifacts_path(
            &Utf8PathBuf::from_path_buf(temp.to_path_buf().join("target")).unwrap(),
            "basic_package",
            "dev",
        );
        let path = result.unwrap();
        assert!(path.is_none());
    }

    #[test]
    fn parsing_starknet_artifacts() {
        let temp = setup_package("basic_package");

        ScarbCommand::new_with_stdio()
            .current_dir(temp.path())
            .arg("build")
            .run()
            .unwrap();

        let artifacts_path = temp
            .path()
            .join("target/dev/basic_package.starknet_artifacts.json");
        let artifacts_path = Utf8PathBuf::from_path_buf(artifacts_path).unwrap();

        let artifacts = artifacts_for_package(&artifacts_path).unwrap();

        assert!(!artifacts.contracts.is_empty());
    }

    #[test]
    fn parsing_starknet_artifacts_on_invalid_file() {
        let temp = TempDir::new().unwrap();
        temp.copy_from("../../", &[".tool-versions"]).unwrap();
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
        let temp = setup_package("basic_package");

        ScarbCommand::new_with_stdio()
            .current_dir(temp.path())
            .arg("build")
            .run()
            .unwrap();

        let metadata = MetadataCommand::new()
            .inherit_stderr()
            .manifest_path(temp.join("Scarb.toml"))
            .exec()
            .unwrap();

        let package = metadata.packages.get(0).unwrap();
        let contracts = get_contracts_map(&metadata, &package.id).unwrap();

        assert!(contracts.contains_key("ERC20"));
        assert!(contracts.contains_key("HelloStarknet"));

        let sierra_contents_erc20 =
            fs::read_to_string(temp.join("target/dev/basic_package_ERC20.contract_class.json"))
                .unwrap();
        let casm_contents_erc20 = fs::read_to_string(
            temp.join("target/dev/basic_package_ERC20.compiled_contract_class.json"),
        )
        .unwrap();
        let contract = contracts.get("ERC20").unwrap();
        assert_eq!(&sierra_contents_erc20, &contract.sierra);
        assert_eq!(&casm_contents_erc20, &contract.casm);

        let sierra_contents_erc20 = fs::read_to_string(
            temp.join("target/dev/basic_package_HelloStarknet.contract_class.json"),
        )
        .unwrap();
        let casm_contents_erc20 = fs::read_to_string(
            temp.join("target/dev/basic_package_HelloStarknet.compiled_contract_class.json"),
        )
        .unwrap();
        let contract = contracts.get("HelloStarknet").unwrap();
        assert_eq!(&sierra_contents_erc20, &contract.sierra);
        assert_eq!(&casm_contents_erc20, &contract.casm);
    }

    #[test]
    fn get_name_for_package() {
        let temp = setup_package("basic_package");
        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let package_name =
            name_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0]).unwrap();

        assert_eq!(package_name, "basic_package".to_string());
    }

    #[test]
    fn get_target_name_for_package() {
        let temp = setup_package("basic_package");
        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let target_name =
            target_name_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0]).unwrap();

        assert_eq!(target_name, "basic_package");
    }
}
