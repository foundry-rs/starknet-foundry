use crate::e2e::common::runner::{get_current_branch, get_remote_url, setup_package};
use crate::utils::use_snforge_std_deprecated;
use assert_fs::fixture::{FileWriteStr, PathChild};
use indoc::formatdoc;
use scarb_api::ScarbCommand;
use shared::test_utils::output_assert::AsOutput;
use snapbox::cmd::Command as SnapboxCommand;

#[test]
fn no_warnings_are_produced() {
    let temp = setup_package("simple_package");

    let remote_url = get_remote_url().to_lowercase();
    let branch = get_current_branch();
    let manifest_path = temp.child("Scarb.toml");

    let snforge_std = if use_snforge_std_deprecated() {
        format!(
            r#"snforge_std_deprecated = {{ git = "https://github.com/{remote_url}", branch = "{branch}" }}"#
        )
    } else {
        format!(
            r#"snforge_std = {{ git = "https://github.com/{remote_url}", branch = "{branch}" }}"#
        )
    };

    manifest_path
        .write_str(&formatdoc!(
            r#"
            [package]
            name = "simple_package"
            version = "0.1.0"
            edition = "2024_07"

            [[target.starknet-contract]]

            [dependencies]
            starknet = "2.10.1"
            {snforge_std}

            [cairo]
            allow-warnings = false
            "#,
        ))
        .unwrap();

    let output = SnapboxCommand::from(
        ScarbCommand::new()
            .current_dir(temp.path())
            .args(["build", "--test"])
            .command(),
    )
    .assert()
    .code(0);

    assert!(!output.as_stdout().contains("warn:"));
}
