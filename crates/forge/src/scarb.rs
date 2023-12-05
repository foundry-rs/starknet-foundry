use scarb_metadata::{Metadata, PackageId};
use std::process::{Command, Stdio};

use crate::scarb::config::{ForgeConfig, RawForgeConfig};
use anyhow::{anyhow, Context, Result};
use scarb_ui::args::PackagesFilter;

pub mod config;

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
    let maybe_raw_metadata = metadata
        .get_package(package)
        .ok_or_else(|| anyhow!("Failed to find metadata for package = {package}"))?
        .tool_metadata("snforge");
    let raw_config = if let Some(raw_metadata) = maybe_raw_metadata {
        serde_json::from_value::<RawForgeConfig>(raw_metadata.clone())?
    } else {
        Default::default()
    };

    raw_config
        .try_into()
        .context("Invalid config in Scarb.toml: ")
}

pub fn build_contracts_with_scarb(filter: PackagesFilter) -> Result<()> {
    let build_output = Command::new("scarb")
        .arg("build")
        .env("SCARB_PACKAGES_FILTER", filter.to_env())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .context("Failed to build contracts with Scarb")?;

    if build_output.status.success() {
        Ok(())
    } else {
        Err(anyhow!("scarb build did not succeed"))
    }
}

pub fn build_test_artifacts_with_scarb(filter: PackagesFilter) -> Result<()> {
    let build_output = Command::new("scarb")
        .arg("snforge-test-collector")
        .env("SCARB_PACKAGES_FILTER", filter.to_env())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .context("Failed to build test artifacts with Scarb")?;

    if build_output.status.success() {
        Ok(())
    } else {
        Err(anyhow!("scarb snforge-test-collector did not succeed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scarb::config::ForkTarget;
    use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
    use assert_fs::TempDir;
    use camino::Utf8PathBuf;
    use indoc::{formatdoc, indoc};
    use scarb_metadata::MetadataCommand;
    use std::str::FromStr;
    use test_collector::RawForkParams;

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
                starknet = "2.3.1"
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
                    ForkTarget::new(
                        "FIRST_FORK_NAME".to_string(),
                        RawForkParams {
                            url: "http://some.rpc.url".to_string(),
                            block_id_type: "number".to_string(),
                            block_id_value: "1".to_string(),
                        },
                    ),
                    ForkTarget::new(
                        "SECOND_FORK_NAME".to_string(),
                        RawForkParams {
                            url: "http://some.rpc.url".to_string(),
                            block_id_type: "hash".to_string(),
                            block_id_value: "1".to_string(),
                        },
                    ),
                    ForkTarget::new(
                        "THIRD_FORK_NAME".to_string(),
                        RawForkParams {
                            url: "http://some.rpc.url".to_string(),
                            block_id_type: "tag".to_string(),
                            block_id_value: "Latest".to_string(),
                        },
                    )
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

        assert!(format!("{err:?}").contains("Some fork names are duplicated"));
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
        assert!(format!("{err:?}").contains("block_id should be set once per fork"));
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
        assert!(
            format!("{err:?}").contains("block_id = wrong_variant is not valid. Possible values are = \"number\", \"hash\" and \"tag\"")
        );
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
            block_id.tag = "Pending"
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
        assert!(format!("{err:?}").contains("block_id.tag can only be equal to Latest"));
    }
}
