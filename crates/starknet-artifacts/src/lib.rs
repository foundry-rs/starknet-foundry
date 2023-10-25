use anyhow::{anyhow, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use scarb_metadata::{CompilationUnitMetadata, Metadata, MetadataCommand, PackageId};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use test_collector::LinkedLibrary;

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
    casm: Utf8PathBuf,
}

/// Contains compiled Starknet artifacts
#[derive(Debug, PartialEq, Clone)]
pub struct StarknetContractArtifacts {
    /// Compiled sierra code
    pub sierra: String,
    /// Compiled casm code
    pub casm: String,
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
            .with_context(|| format!("Failed to parse {path:?} contents. Make sure you have enabled sierra and casm code generation in Scarb.toml"))?;
    Ok(starknet_artifacts)
}

/// Try getting path to Scarb starknet artifacts for the given package
///
/// Try getting the path to `starknet_artifacts.json` file that is generated by `scarb build` command.
/// If the file is not present, for example when package doesn't define the starknet target, `None` is returned.
///
/// # Arguments
///
/// * `path` - A path to the Scarb package
/// * `target_name` - A name of the target that is being built by Scarb
pub fn try_get_starknet_artifacts_path(
    target_dir: &Utf8Path,
    target_name: &str,
) -> Result<Option<Utf8PathBuf>> {
    let path = target_dir.join("dev");
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
///
/// # Arguments
///
/// * path - A path to the Scarb package
pub fn get_contracts_map(path: &Utf8Path) -> Result<HashMap<String, StarknetContractArtifacts>> {
    let base_path = path
        .parent()
        .ok_or_else(|| anyhow!("Failed to get parent for path = {}", path))?;
    let artifacts = artifacts_for_package(path)?;
    let mut map = HashMap::new();
    for contract in artifacts.contracts {
        let name = contract.contract_name;
        let sierra_path = base_path.join(contract.artifacts.sierra);
        let casm_path = base_path.join(contract.artifacts.casm);
        let sierra = fs::read_to_string(sierra_path)?;
        let casm = fs::read_to_string(casm_path)?;
        map.insert(name, StarknetContractArtifacts { sierra, casm });
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
        .min_by_key(|unit| match unit.target.name.as_str() {
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

/// Get the path to Cairo corelib for the given package
pub fn corelib_for_package(metadata: &Metadata, package: &PackageId) -> Result<Utf8PathBuf> {
    let compilation_unit = compilation_unit_for_package(metadata, package)?;
    let corelib = compilation_unit
        .components
        .iter()
        .find(|du| du.name == "core")
        .context("corelib could not be found")?;
    Ok(Utf8PathBuf::from(corelib.source_root()))
}

/// Get the top-level and main file paths for the given package
pub fn paths_for_package(
    metadata: &Metadata,
    package: &PackageId,
) -> Result<(Utf8PathBuf, Utf8PathBuf)> {
    let compilation_unit = compilation_unit_for_package(metadata, package)?;

    let package = metadata
        .get_package(package)
        .ok_or_else(|| anyhow!("Failed to find metadata for package = {package}"))?;

    let package_path = package.root.clone();
    let package_source_dir_path = compilation_unit.target.source_root();

    Ok((package_path, Utf8PathBuf::from(package_source_dir_path)))
}

pub fn target_dir_for_package(workspace_root: &Utf8Path) -> Result<Utf8PathBuf> {
    let scarb_metadata = MetadataCommand::new().inherit_stderr().exec()?;

    let target_dir = scarb_metadata
        .target_dir
        .unwrap_or_else(|| workspace_root.join("target"));

    Ok(target_dir)
}

/// Get a name of the given package
pub fn name_for_package(metadata: &Metadata, package: &PackageId) -> Result<String> {
    let package = metadata
        .get_package(package)
        .ok_or_else(|| anyhow!("Failed to find metadata for package = {package}"))?;

    Ok(package.name.clone())
}

/// Get the dependencies for the given package
pub fn dependencies_for_package(
    metadata: &Metadata,
    package: &PackageId,
) -> Result<Vec<LinkedLibrary>> {
    let compilation_unit = compilation_unit_for_package(metadata, package)?;
    let dependencies = compilation_unit
        .components
        .iter()
        .filter(|du| &du.name != "core")
        .map(|cu| LinkedLibrary {
            name: cu.name.clone(),
            path: cu.source_root().to_owned().into_std_path_buf(),
        })
        .collect();

    Ok(dependencies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::fixture::{FileTouch, FileWriteStr, PathChild, PathCopy};
    use assert_fs::TempDir;
    use indoc::{formatdoc, indoc};
    use scarb_metadata::MetadataCommand;
    use std::process::Command;
    use std::str::FromStr;

    fn setup_package(package_name: &str) -> TempDir {
        let temp = TempDir::new().unwrap();
        temp.copy_from(
            format!("tests/data/{package_name}"),
            &["**/*.cairo", "**/*.toml"],
        )
        .unwrap();

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
                starknet = "2.2.0"
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
        let temp = setup_package("simple_package");

        let build_output = Command::new("scarb")
            .current_dir(&temp)
            .arg("build")
            .output()
            .unwrap();
        assert!(build_output.status.success());

        let result = try_get_starknet_artifacts_path(
            &Utf8PathBuf::from_path_buf(temp.to_path_buf().join("target")).unwrap(),
            "simple_package",
        );
        let path = result.unwrap().unwrap();
        assert_eq!(
            path,
            temp.path()
                .join("target/dev/simple_package.starknet_artifacts.json")
        );
    }

    #[test]
    fn get_starknet_artifacts_path_for_project_with_different_package_and_target_name() {
        let temp = setup_package("simple_package");

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
                name = "simple_package"
                version = "0.1.0"

                [dependencies]
                starknet = "2.2.0"
                snforge_std = {{ path = "{}" }}

                [[target.starknet-contract]]
                name = "essa"
                sierra = true
                casm = true
                "#,
                snforge_std_path
            ))
            .unwrap();

        let build_output = Command::new("scarb")
            .current_dir(&temp)
            .arg("build")
            .output()
            .unwrap();
        assert!(build_output.status.success());

        let result = try_get_starknet_artifacts_path(
            &Utf8PathBuf::from_path_buf(temp.to_path_buf().join("target")).unwrap(),
            "essa",
        );
        let path = result.unwrap().unwrap();
        assert_eq!(
            path,
            temp.path().join("target/dev/essa.starknet_artifacts.json")
        );
    }

    #[test]
    fn get_starknet_artifacts_path_for_project_without_starknet_target() {
        let temp = setup_package("panic_decoding");

        let manifest_path = temp.child("Scarb.toml");
        manifest_path
            .write_str(indoc!(
                r#"
            [package]
            name = "panic_decoding"
            version = "0.1.0"
            "#,
            ))
            .unwrap();

        let build_output = Command::new("scarb")
            .current_dir(&temp)
            .arg("build")
            .output()
            .unwrap();
        assert!(build_output.status.success());

        let result = try_get_starknet_artifacts_path(
            &Utf8PathBuf::from_path_buf(temp.to_path_buf().join("target")).unwrap(),
            "panic_decoding",
        );
        let path = result.unwrap();
        assert!(path.is_none());
    }

    #[test]
    fn get_starknet_artifacts_path_for_project_without_scarb_build() {
        let temp = setup_package("simple_package");

        let result = try_get_starknet_artifacts_path(
            &Utf8PathBuf::from_path_buf(temp.to_path_buf().join("target")).unwrap(),
            "simple_package",
        );
        let path = result.unwrap();
        assert!(path.is_none());
    }

    #[test]
    fn parsing_starknet_artifacts() {
        let temp = setup_package("simple_package");

        let build_output = Command::new("scarb")
            .current_dir(&temp)
            .arg("build")
            .output()
            .unwrap();
        assert!(build_output.status.success());

        let artifacts_path = temp
            .path()
            .join("target/dev/simple_package.starknet_artifacts.json");
        let artifacts_path = Utf8PathBuf::from_path_buf(artifacts_path).unwrap();

        let artifacts = artifacts_for_package(&artifacts_path).unwrap();

        assert!(!artifacts.contracts.is_empty());
    }

    #[test]
    fn parsing_starknet_artifacts_on_invalid_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.child("wrong.json");
        path.touch().unwrap();
        path.write_str("\"aa\": {}").unwrap();
        let artifacts_path = Utf8PathBuf::from_path_buf(path.to_path_buf()).unwrap();

        let result = artifacts_for_package(&artifacts_path);
        let err = result.unwrap_err();

        assert!(err.to_string().contains(&format!("Failed to parse {artifacts_path:?} contents. Make sure you have enabled sierra and casm code generation in Scarb.toml")));
    }

    #[test]
    fn get_contracts() {
        let temp = setup_package("simple_package");

        let build_output = Command::new("scarb")
            .current_dir(&temp)
            .arg("build")
            .output()
            .unwrap();
        assert!(build_output.status.success());

        let artifacts_path = temp
            .path()
            .join("target/dev/simple_package.starknet_artifacts.json");
        let artifacts_path = Utf8PathBuf::from_path_buf(artifacts_path).unwrap();

        let contracts = get_contracts_map(&artifacts_path).unwrap();

        assert!(contracts.contains_key("ERC20"));
        assert!(contracts.contains_key("HelloStarknet"));

        let sierra_contents_erc20 =
            fs::read_to_string(temp.join("target/dev/simple_package_ERC20.sierra.json")).unwrap();
        let casm_contents_erc20 =
            fs::read_to_string(temp.join("target/dev/simple_package_ERC20.casm.json")).unwrap();
        let contract = contracts.get("ERC20").unwrap();
        assert_eq!(&sierra_contents_erc20, &contract.sierra);
        assert_eq!(&casm_contents_erc20, &contract.casm);

        let sierra_contents_erc20 =
            fs::read_to_string(temp.join("target/dev/simple_package_HelloStarknet.sierra.json"))
                .unwrap();
        let casm_contents_erc20 =
            fs::read_to_string(temp.join("target/dev/simple_package_HelloStarknet.casm.json"))
                .unwrap();
        let contract = contracts.get("HelloStarknet").unwrap();
        assert_eq!(&sierra_contents_erc20, &contract.sierra);
        assert_eq!(&casm_contents_erc20, &contract.casm);
    }

    #[test]
    fn get_dependencies_for_package() {
        let temp = setup_package("simple_package");
        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let dependencies =
            dependencies_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0])
                .unwrap();

        assert!(!dependencies.is_empty());
        assert!(dependencies.iter().all(|dep| dep.path.exists()));
    }

    #[test]
    fn get_paths_for_package() {
        let temp = setup_package("simple_package");
        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let (package_path, package_source_dir_path) =
            paths_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0]).unwrap();

        assert!(package_path.is_dir());
        assert!(package_source_dir_path.is_dir());
        assert_eq!(package_source_dir_path, package_path.join("src"));
        assert!(package_source_dir_path.starts_with(package_path));
    }

    #[test]
    fn get_name_for_package() {
        let temp = setup_package("simple_package");
        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let package_name =
            name_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0]).unwrap();

        assert_eq!(package_name, "simple_package".to_string());
    }

    #[test]
    fn get_corelib_path_for_package() {
        let temp = setup_package("simple_package");
        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let corelib_path =
            corelib_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0]).unwrap();

        assert!(corelib_path.is_dir());
        assert!(corelib_path.exists());

        let lib_path = corelib_path.join("lib.cairo");
        assert!(lib_path.exists());
    }

    #[test]
    fn get_target_name_for_package() {
        let temp = setup_package("simple_package");
        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let target_name =
            target_name_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0]).unwrap();

        assert_eq!(target_name, "simple_package");
    }

    #[test]
    fn get_dependencies_for_package_err_on_invalid_package() {
        let temp = setup_package("simple_package");
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
}
