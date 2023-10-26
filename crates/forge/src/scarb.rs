use anyhow::{anyhow, Result};
use itertools::Itertools;
use scarb_metadata::{Metadata, PackageId};
use serde::Deserialize;
use std::collections::HashMap;

/// Represents forge config deserialized from Scarb.toml
#[derive(Deserialize, Debug, PartialEq, Default)]
pub struct ForgeConfig {
    #[serde(default)]
    /// Should runner exit after first failed test
    pub exit_first: bool,
    /// How many runs should fuzzer execute
    pub fuzzer_runs: Option<u32>,
    /// Seed to be used by fuzzer
    pub fuzzer_seed: Option<u64>,

    #[serde(default)]
    pub fork: Vec<ForkTarget>,
}

#[derive(Deserialize, Debug, PartialEq, Default, Clone)]
pub struct ForkTarget {
    pub name: String,
    pub url: String,
    pub block_id: HashMap<String, String>,
}

/// Get Forge config from the `Scarb.toml` file
///
/// # Arguments
///
/// * `metadata` - Scarb metadata object
/// * `package` - Id of the Scarb package
pub fn config_from_scarb_for_package(
    metadata: &Metadata,
    package: &PackageId,
) -> Result<ForgeConfig> {
    let raw_metadata = metadata
        .get_package(package)
        .ok_or_else(|| anyhow!("Failed to find metadata for package = {package}"))?
        .tool_metadata("snforge");
    let config = raw_metadata.map_or_else(
        || Ok(Default::default()),
        |raw_metadata| Ok(serde_json::from_value(raw_metadata.clone())?),
    );
    validate_fork_config(config)
}

fn validate_fork_config(config: Result<ForgeConfig>) -> Result<ForgeConfig> {
    if let Ok(ForgeConfig { fork: forks, .. }) = &config {
        let names: Vec<String> = forks.iter().map(|fork| fork.name.clone()).collect();
        let removed_duplicated_names: Vec<String> = names.clone().into_iter().unique().collect();

        if names.len() != removed_duplicated_names.len() {
            return Err(anyhow!("Some fork names are duplicated"));
        }

        for fork in forks {
            let block_id_items: Vec<(&String, &String)> = fork.block_id.iter().collect();
            let [(block_id_key, block_id_value)] = block_id_items[..] else {
                return Err(anyhow!("block_id should be set once per fork"));
            };

            if !["number", "hash", "tag"].contains(&&**block_id_key) {
                return Err(anyhow!(
                    "block_id has only three variants: number, hash and tag"
                ));
            }

            if block_id_key == "tag" && !["Latest", "Pending"].contains(&&**block_id_value) {
                return Err(anyhow!(
                    "block_id.tag has only two variants: Latest or Pending"
                ));
            }
        }
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
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
                starknet = "2.3.0"
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
                starknet = "2.3.0"
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
            fs::read_to_string(temp.join("target/dev/simple_package_ERC20.contract_class.json"))
                .unwrap();
        let casm_contents_erc20 = fs::read_to_string(
            temp.join("target/dev/simple_package_ERC20.compiled_contract_class.json"),
        )
        .unwrap();
        let contract = contracts.get("ERC20").unwrap();
        assert_eq!(&sierra_contents_erc20, &contract.sierra);
        assert_eq!(&casm_contents_erc20, &contract.casm);

        let sierra_contents_erc20 = fs::read_to_string(
            temp.join("target/dev/simple_package_HelloStarknet.contract_class.json"),
        )
        .unwrap();
        let casm_contents_erc20 = fs::read_to_string(
            temp.join("target/dev/simple_package_HelloStarknet.compiled_contract_class.json"),
        )
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

    #[test]
    fn get_forge_config_for_package() {
        let temp = setup_package("simple_package");
        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let config =
            config_from_scarb_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0])
                .unwrap();

        assert_eq!(
            config,
            ForgeConfig {
                exit_first: false,
                fork: vec![
                    ForkTarget {
                        name: "FIRST_FORK_NAME".to_string(),
                        url: "http://some.rpc.url".to_string(),
                        block_id: HashMap::from([("number".to_string(), "1".to_string())]),
                    },
                    ForkTarget {
                        name: "SECOND_FORK_NAME".to_string(),
                        url: "http://some.rpc.url".to_string(),
                        block_id: HashMap::from([("hash".to_string(), "1".to_string())]),
                    },
                    ForkTarget {
                        name: "THIRD_FORK_NAME".to_string(),
                        url: "http://some.rpc.url".to_string(),
                        block_id: HashMap::from([("tag".to_string(), "Latest".to_string())]),
                    }
                ],
                fuzzer_runs: None,
                fuzzer_seed: None,
            }
        );
    }

    #[test]
    fn get_forge_config_for_package_err_on_invalid_package() {
        let temp = setup_package("simple_package");
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
        let temp = setup_package("simple_package");
        let content = indoc!(
            r#"
            [package]
            name = "simple_package"
            version = "0.1.0"
            "#
        );
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

    #[test]
    fn get_forge_config_for_package_fails_on_same_fork_name() {
        let temp = setup_package("simple_package");
        let content = indoc!(
            r#"
            [package]
            name = "simple_package"
            version = "0.1.0"

            [[tool.snforge.fork]]
            name = "SAME_NAME"
            url = "http://some.rpc.url"
            block_id.number = "1"

            [[tool.snforge.fork]]
            name = "SAME_NAME"
            url = "http://some.rpc.url"
            block_id.hash = "1"
            "#
        );
        temp.child("Scarb.toml").write_str(content).unwrap();

        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();
        let err =
            config_from_scarb_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0])
                .unwrap_err();

        assert!(err.to_string().contains("Some fork names are duplicated"));
    }

    #[test]
    fn get_forge_config_for_package_fails_on_multiple_block_id() {
        let temp = setup_package("simple_package");
        let content = indoc!(
            r#"
            [package]
            name = "simple_package"
            version = "0.1.0"

            [[tool.snforge.fork]]
            name = "SAME_NAME"
            url = "http://some.rpc.url"
            block_id.number = "1"
            block_id.hash = "2"
            "#
        );
        temp.child("Scarb.toml").write_str(content).unwrap();

        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();
        let err =
            config_from_scarb_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0])
                .unwrap_err();
        assert!(err
            .to_string()
            .contains("block_id should be set once per fork"));
    }

    #[test]
    fn get_forge_config_for_package_fails_on_wrong_block_id() {
        let temp = setup_package("simple_package");
        let content = indoc!(
            r#"
            [package]
            name = "simple_package"
            version = "0.1.0"

            [[tool.snforge.fork]]
            name = "SAME_NAME"
            url = "http://some.rpc.url"
            block_id.wrong_variant = "1"
            "#
        );
        temp.child("Scarb.toml").write_str(content).unwrap();

        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let err =
            config_from_scarb_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0])
                .unwrap_err();
        assert!(err
            .to_string()
            .contains("block_id has only three variants: number, hash and tag"));
    }

    #[test]
    fn get_forge_config_for_package_fails_on_wrong_block_tag() {
        let temp = setup_package("simple_package");
        let content = indoc!(
            r#"
            [package]
            name = "simple_package"
            version = "0.1.0"

            [[tool.snforge.fork]]
            name = "SAME_NAME"
            url = "http://some.rpc.url"
            block_id.tag = "Wrong tag"
            "#
        );
        temp.child("Scarb.toml").write_str(content).unwrap();

        let scarb_metadata = MetadataCommand::new()
            .inherit_stderr()
            .current_dir(temp.path())
            .exec()
            .unwrap();

        let err =
            config_from_scarb_for_package(&scarb_metadata, &scarb_metadata.workspace.members[0])
                .unwrap_err();
        assert!(err
            .to_string()
            .contains("block_id.tag has only two variants: Latest or Pending"));
    }
}
