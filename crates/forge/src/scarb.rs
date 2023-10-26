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
