use crate::scarb::config::ForgeConfigFromScarb;
use anyhow::{Context, Result};
use cairo_lang_sierra::program::VersionedProgram;
use camino::Utf8Path;
use configuration::PackageConfig;
use forge_runner::package_tests::TestTargetLocation;
use forge_runner::package_tests::raw::TestTargetRaw;
use scarb_api::{ScarbCommand, test_targets_by_name};
use scarb_metadata::PackageMetadata;
use scarb_ui::args::{FeaturesSpec, PackagesFilter};
use semver::Version;
use std::fs;
use std::io::ErrorKind;

pub mod config;

const MINIMAL_SCARB_VERSION_TO_OPTIMIZE_COMPILATION: Version = Version::new(2, 8, 3);

impl PackageConfig for ForgeConfigFromScarb {
    fn tool_name() -> &'static str {
        "snforge"
    }

    fn from_raw(config: &serde_json::Value) -> Result<Self>
    where
        Self: Sized,
    {
        serde_json::from_value(config.clone()).context("Failed to parse snforge config")
    }
}

#[must_use]
pub fn should_compile_starknet_contract_target(
    scarb_version: &Version,
    no_optimization: bool,
) -> bool {
    *scarb_version < MINIMAL_SCARB_VERSION_TO_OPTIMIZE_COMPILATION || no_optimization
}

pub fn build_artifacts_with_scarb(
    filter: PackagesFilter,
    features: FeaturesSpec,
    scarb_version: &Version,
    no_optimization: bool,
) -> Result<()> {
    if should_compile_starknet_contract_target(scarb_version, no_optimization) {
        build_contracts_with_scarb(filter.clone(), features.clone())?;
    }
    build_test_artifacts_with_scarb(filter, features)?;
    Ok(())
}

fn build_contracts_with_scarb(filter: PackagesFilter, features: FeaturesSpec) -> Result<()> {
    ScarbCommand::new_with_stdio()
        .arg("build")
        .packages_filter(filter)
        .features(features)
        .run()
        .context("Failed to build contracts with Scarb")?;
    Ok(())
}

fn build_test_artifacts_with_scarb(filter: PackagesFilter, features: FeaturesSpec) -> Result<()> {
    ScarbCommand::new_with_stdio()
        .arg("build")
        .arg("--test")
        .packages_filter(filter)
        .features(features)
        .run()
        .context("Failed to build test artifacts with Scarb")?;
    Ok(())
}

pub fn load_test_artifacts(
    target_dir: &Utf8Path,
    package: &PackageMetadata,
) -> Result<Vec<TestTargetRaw>> {
    let mut targets = vec![];

    let dedup_targets = test_targets_by_name(package);

    for (target_name, target) in dedup_targets {
        let tests_location =
            if target.params.get("test-type").and_then(|v| v.as_str()) == Some("unit") {
                TestTargetLocation::Lib
            } else {
                TestTargetLocation::Tests
            };

        let target_file = format!("{target_name}.test.sierra.json");
        let sierra_program_path = target_dir.join(target_file);

        match fs::read_to_string(&sierra_program_path) {
            Ok(value) => {
                let versioned_program = serde_json::from_str::<VersionedProgram>(&value)?;

                let sierra_program = match versioned_program {
                    VersionedProgram::V1 { program, .. } => program,
                };

                let test_target = TestTargetRaw {
                    sierra_program,
                    sierra_program_path,
                    tests_location,
                };

                targets.push(test_target);
            }
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => Err(err)?,
        }
    }

    Ok(targets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scarb::config::ForkTarget;
    use assert_fs::TempDir;
    use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
    use camino::Utf8PathBuf;
    use cheatnet::runtime_extensions::forge_config_extension::config::BlockId;
    use configuration::load_package_config;
    use indoc::{formatdoc, indoc};
    use scarb_api::metadata::MetadataCommandExt;
    use scarb_metadata::PackageId;
    use std::env;
    use std::str::FromStr;
    use test_utils::tempdir_with_tool_versions;

    fn setup_package(package_name: &str) -> TempDir {
        let temp = tempdir_with_tool_versions().unwrap();
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
                block_id.hash = "0xa"

                [[tool.snforge.fork]]
                name = "THIRD_FORK_NAME"
                url = "http://some.rpc.url"
                block_id.hash = "10"

                [[tool.snforge.fork]]
                name = "FOURTH_FORK_NAME"
                url = "http://some.rpc.url"
                block_id.tag = "latest"
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
        let scarb_metadata = ScarbCommand::metadata()
            .inherit_stderr()
            .current_dir(temp.path())
            .run()
            .unwrap();

        let config = load_package_config::<ForgeConfigFromScarb>(
            &scarb_metadata,
            &scarb_metadata.workspace.members[0],
        )
        .unwrap();

        assert_eq!(
            config,
            ForgeConfigFromScarb {
                exit_first: false,
                fork: vec![
                    ForkTarget {
                        name: "FIRST_FORK_NAME".to_string(),
                        url: "http://some.rpc.url".parse().expect("Should be valid url"),
                        block_id: BlockId::BlockNumber(1),
                    },
                    ForkTarget {
                        name: "SECOND_FORK_NAME".to_string(),
                        url: "http://some.rpc.url".parse().expect("Should be valid url"),
                        block_id: BlockId::BlockHash(0xa.into()),
                    },
                    ForkTarget {
                        name: "THIRD_FORK_NAME".to_string(),
                        url: "http://some.rpc.url".parse().expect("Should be valid url"),
                        block_id: BlockId::BlockHash(10.into()),
                    },
                    ForkTarget {
                        name: "FOURTH_FORK_NAME".to_string(),
                        url: "http://some.rpc.url".parse().expect("Should be valid url"),
                        block_id: BlockId::BlockTag,
                    },
                ],
                fuzzer_runs: None,
                fuzzer_seed: None,
                max_n_steps: None,
                detailed_resources: false,
                save_trace_data: false,
                build_profile: false,
                coverage: false,
            }
        );
    }

    #[test]
    fn get_forge_config_for_package_err_on_invalid_package() {
        let temp = setup_package("simple_package");
        let scarb_metadata = ScarbCommand::metadata()
            .inherit_stderr()
            .current_dir(temp.path())
            .run()
            .unwrap();

        let result = load_package_config::<ForgeConfigFromScarb>(
            &scarb_metadata,
            &PackageId::from(String::from("12345679")),
        );
        let err = result.unwrap_err();

        assert!(
            err.to_string()
                .contains("Failed to find metadata for package")
        );
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

        let scarb_metadata = ScarbCommand::metadata()
            .inherit_stderr()
            .current_dir(temp.path())
            .run()
            .unwrap();

        let config = load_package_config::<ForgeConfigFromScarb>(
            &scarb_metadata,
            &scarb_metadata.workspace.members[0],
        )
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

        let scarb_metadata = ScarbCommand::metadata()
            .inherit_stderr()
            .current_dir(temp.path())
            .run()
            .unwrap();
        let err = load_package_config::<ForgeConfigFromScarb>(
            &scarb_metadata,
            &scarb_metadata.workspace.members[0],
        )
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

        let scarb_metadata = ScarbCommand::metadata()
            .inherit_stderr()
            .current_dir(temp.path())
            .run()
            .unwrap();
        let err = load_package_config::<ForgeConfigFromScarb>(
            &scarb_metadata,
            &scarb_metadata.workspace.members[0],
        )
        .unwrap_err();
        assert!(
            format!("{err:?}")
                .contains("block_id must contain exactly one key: 'tag', 'hash', or 'number'")
        );
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

        let scarb_metadata = ScarbCommand::metadata()
            .inherit_stderr()
            .current_dir(temp.path())
            .run()
            .unwrap();

        let err = load_package_config::<ForgeConfigFromScarb>(
            &scarb_metadata,
            &scarb_metadata.workspace.members[0],
        )
        .unwrap_err();
        assert!(
            format!("{err:?}")
                .contains("unknown field `wrong_variant`, expected one of `tag`, `hash`, `number`")
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

        let scarb_metadata = ScarbCommand::metadata()
            .inherit_stderr()
            .current_dir(temp.path())
            .run()
            .unwrap();

        let err = load_package_config::<ForgeConfigFromScarb>(
            &scarb_metadata,
            &scarb_metadata.workspace.members[0],
        )
        .unwrap_err();
        assert!(format!("{err:?}").contains("block_id.tag can only be equal to latest"));
    }

    #[test]
    fn get_forge_config_for_package_with_block_tag() {
        let temp = setup_package("simple_package");
        let content = indoc!(
            r#"
            [package]
            name = "simple_package"
            version = "0.1.0"

            [[tool.snforge.fork]]
            name = "SAME_NAME"
            url = "http://some.rpc.url"
            block_id.tag = "latest"
            "#
        );
        temp.child("Scarb.toml").write_str(content).unwrap();

        let scarb_metadata = ScarbCommand::metadata()
            .inherit_stderr()
            .current_dir(temp.path())
            .run()
            .unwrap();

        let forge_config = load_package_config::<ForgeConfigFromScarb>(
            &scarb_metadata,
            &scarb_metadata.workspace.members[0],
        )
        .unwrap();
        assert_eq!(forge_config.fork[0].block_id, BlockId::BlockTag);
    }

    #[test]
    fn get_forge_config_resolves_env_variables() {
        let temp = setup_package("simple_package");
        let content = indoc!(
            r#"
            [package]
            name = "simple_package"
            version = "0.1.0"

            [[tool.snforge.fork]]
            name = "ENV_URL_FORK"
            url = "$ENV_URL_FORK234980670176"
            block_id.number = "1"
            "#
        );
        temp.child("Scarb.toml").write_str(content).unwrap();

        let scarb_metadata = ScarbCommand::metadata()
            .inherit_stderr()
            .current_dir(temp.path())
            .run()
            .unwrap();

        // SAFETY: This value is only read here and is not modified by other tests.
        unsafe {
            env::set_var("ENV_URL_FORK234980670176", "http://some.rpc.url_from_env");
        }
        let config = load_package_config::<ForgeConfigFromScarb>(
            &scarb_metadata,
            &scarb_metadata.workspace.members[0],
        )
        .unwrap();

        assert_eq!(
            config,
            ForgeConfigFromScarb {
                exit_first: false,
                fork: vec![ForkTarget {
                    name: "ENV_URL_FORK".to_string(),
                    url: "http://some.rpc.url_from_env"
                        .parse()
                        .expect("Should be valid url"),
                    block_id: BlockId::BlockNumber(1),
                }],
                fuzzer_runs: None,
                fuzzer_seed: None,
                max_n_steps: None,
                detailed_resources: false,
                save_trace_data: false,
                build_profile: false,
                coverage: false,
            }
        );
    }
}
