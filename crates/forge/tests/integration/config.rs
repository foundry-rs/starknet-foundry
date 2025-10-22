use crate::e2e::common::runner::setup_package;
use assert_fs::TempDir;
use assert_fs::fixture::{FileWriteStr, PathChild};
use cheatnet::runtime_extensions::forge_config_extension::config::BlockId;
use forge::scarb::config::{ForgeConfigFromScarb, ForkTarget};
use forge::scarb::load_package_config;
use forge_runner::forge_config::ForgeTrackedResource;
use indoc::{formatdoc, indoc};
use scarb_api::ScarbCommand;
use scarb_api::metadata::MetadataCommandExt;
use scarb_metadata::PackageId;
use std::{env, fs};

fn setup_package_with_toml() -> TempDir {
    let temp = setup_package("simple_package");

    let manifest_path = temp.child("Scarb.toml");
    let manifest_contents = fs::read_to_string(&manifest_path).unwrap();
    let manifest_contents = formatdoc!(
        r#"
        {manifest_contents}

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
    "#
    );
    manifest_path.write_str(manifest_contents.as_str()).unwrap();

    temp
}

#[test]
fn get_forge_config_for_package() {
    let temp = setup_package_with_toml();
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
            tracked_resource: ForgeTrackedResource::SierraGas,
            detailed_resources: false,
            save_trace_data: false,
            build_profile: false,
            coverage: false,
        }
    );
}

#[test]
fn get_forge_config_for_package_err_on_invalid_package() {
    let temp = setup_package_with_toml();
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
    let temp = setup_package_with_toml();
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

    assert_eq!(config, ForgeConfigFromScarb::default());
}

#[test]
fn get_forge_config_for_package_fails_on_same_fork_name() {
    let temp = setup_package_with_toml();
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
    let temp = setup_package_with_toml();
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
    let temp = setup_package_with_toml();
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
    let temp = setup_package_with_toml();
    let content = indoc!(
        r#"
            [package]
            name = "simple_package"
            version = "0.1.0"

            [[tool.snforge.fork]]
            name = "SAME_NAME"
            url = "http://some.rpc.url"
            block_id.tag = "Preconfirmed"
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
    let temp = setup_package_with_toml();
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
    let temp = setup_package_with_toml();
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
            tracked_resource: ForgeTrackedResource::SierraGas,
            detailed_resources: false,
            save_trace_data: false,
            build_profile: false,
            coverage: false,
        }
    );
}
