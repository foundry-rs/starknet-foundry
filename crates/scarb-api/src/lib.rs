use anyhow::{anyhow, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use scarb_metadata::{CompilationUnitMetadata, Metadata, PackageId};
use semver::VersionReq;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use universal_sierra_compiler_api::{compile_sierra_at_path, SierraType};

pub use command::*;

mod command;
pub mod metadata;
pub mod version;

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

        let casm = compile_sierra_at_path(
            starknet_contract.artifacts.sierra.as_str(),
            Some(base_path.as_std_path()),
            &SierraType::Contract,
        )?;

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
pub fn get_contracts_artifacts_and_source_sierra_paths(
    metadata: &Metadata,
    package: &PackageId,
    profile: Option<&str>,
) -> Result<HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)>> {
    let target_name = target_name_for_package(metadata, package)?;
    let target_dir = target_dir_for_workspace(metadata);
    let maybe_contracts_path = try_get_starknet_artifacts_path(
        &target_dir,
        &target_name,
        profile.unwrap_or(metadata.current_profile.as_str()),
    )?;

    let map = match maybe_contracts_path {
        Some(contracts_path) => load_contracts_artifacts_and_source_sierra_paths(&contracts_path)?,
        None => HashMap::default(),
    };

    Ok(map)
}

fn load_contracts_artifacts_and_source_sierra_paths(
    contracts_path: &Utf8PathBuf,
) -> Result<HashMap<String, (StarknetContractArtifacts, Utf8PathBuf)>> {
    let base_path = contracts_path
        .parent()
        .ok_or_else(|| anyhow!("Failed to get parent for path = {}", &contracts_path))?;
    let artifacts = artifacts_for_package(contracts_path)?;
    let mut map = HashMap::new();

    for ref contract in artifacts.contracts {
        let name = contract.contract_name.clone();
        let contract_artifacts =
            StarknetContractArtifacts::from_scarb_contract_artifact(contract, base_path)?;

        let sierra_path = base_path.join(contract.artifacts.sierra.clone());

        map.insert(name.clone(), (contract_artifacts, sierra_path));
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

/// Checks if the specified package has version compatible with the specified requirement
pub fn package_matches_version_requirement(
    metadata: &Metadata,
    name: &str,
    version_req: &VersionReq,
) -> Result<bool> {
    let mut packages = metadata
        .packages
        .iter()
        .filter(|package| package.name == name);

    match (packages.next(), packages.next()) {
        (Some(package), None) => Ok(version_req.matches(&package.version)),
        (None, None) => Err(anyhow!("Package {name} is not present in dependencies.")),
        _ => Err(anyhow!("Package {name} is duplicated in dependencies")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::MetadataCommandExt;
    use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
    use assert_fs::prelude::FileTouch;
    use assert_fs::TempDir;
    use camino::Utf8PathBuf;
    use indoc::{formatdoc, indoc};
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

                [dependencies]
                starknet = "2.4.0"
                snforge_std = {{ path = "{}" }}

                [[target.starknet-contract]]

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
    fn package_matches_version_requirement_test() {
        let temp = setup_package("basic_package");

        let manifest_path = temp.child("Scarb.toml");
        manifest_path
            .write_str(&formatdoc!(
                r#"
                [package]
                name = "version_checker"
                version = "0.1.0"

                [[target.starknet-contract]]
                sierra = true

                [dependencies]
                starknet = "2.5.4"
                "#,
            ))
            .unwrap();

        let scarb_metadata = ScarbCommand::metadata()
            .inherit_stderr()
            .current_dir(temp.path())
            .run()
            .unwrap();

        assert!(package_matches_version_requirement(
            &scarb_metadata,
            "starknet",
            &VersionReq::parse("2.5").unwrap(),
        )
        .unwrap());

        assert!(package_matches_version_requirement(
            &scarb_metadata,
            "not_existing",
            &VersionReq::parse("2.5").unwrap(),
        )
        .is_err());

        assert!(!package_matches_version_requirement(
            &scarb_metadata,
            "starknet",
            &VersionReq::parse("2.8").unwrap(),
        )
        .unwrap());
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

        let metadata = ScarbCommand::metadata()
            .inherit_stderr()
            .manifest_path(temp.join("Scarb.toml"))
            .run()
            .unwrap();

        let package = metadata.packages.first().unwrap();
        let contracts =
            get_contracts_artifacts_and_source_sierra_paths(&metadata, &package.id, None).unwrap();

        assert!(contracts.contains_key("ERC20"));
        assert!(contracts.contains_key("HelloStarknet"));

        let sierra_contents_erc20 =
            fs::read_to_string(temp.join("target/dev/basic_package_ERC20.contract_class.json"))
                .unwrap();

        let contract = contracts.get("ERC20").unwrap();
        assert_eq!(&sierra_contents_erc20, &contract.0.sierra);
        assert!(!contract.0.casm.is_empty());

        let sierra_contents_erc20 = fs::read_to_string(
            temp.join("target/dev/basic_package_HelloStarknet.contract_class.json"),
        )
        .unwrap();
        let contract = contracts.get("HelloStarknet").unwrap();
        assert_eq!(&sierra_contents_erc20, &contract.0.sierra);
        assert!(!contract.0.casm.is_empty());
    }

    #[test]
    fn get_name_for_package() {
        let temp = setup_package("basic_package");
        let scarb_metadata = ScarbCommand::metadata()
            .inherit_stderr()
            .current_dir(temp.path())
            .run()
            .unwrap();

        let package_name =
            name_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0]).unwrap();

        assert_eq!(&package_name, "basic_package");
    }

    #[test]
    fn get_target_name_for_package() {
        let temp = setup_package("basic_package");
        let scarb_metadata = ScarbCommand::metadata()
            .inherit_stderr()
            .current_dir(temp.path())
            .run()
            .unwrap();

        let target_name =
            target_name_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0]).unwrap();

        assert_eq!(target_name, "basic_package");
    }
}
