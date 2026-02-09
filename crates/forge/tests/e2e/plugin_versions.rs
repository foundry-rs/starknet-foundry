use crate::e2e::common::runner::{runner, test_runner};
use crate::utils::tempdir_with_tool_versions;
use camino::Utf8PathBuf;
use indoc::{formatdoc, indoc};
use scarb_api::ScarbCommand;
use shared::test_utils::output_assert::assert_stdout_contains;
use snapbox::cmd::Command;
use toml_edit::Document;

#[test]
#[cfg_attr(
    not(feature = "test_for_multiple_scarb_versions"),
    ignore = "Multiple scarb versions must be installed"
)]
fn new_with_new_scarb() {
    let temp = tempdir_with_tool_versions().unwrap();
    runner(&temp)
        .env("DEV_USE_OFFLINE_MODE", "true")
        .args(["new", "abc"])
        .assert()
        .success();

    let manifest = temp.path().join("abc").join("Scarb.toml");
    let manifest = &std::fs::read_to_string(manifest).unwrap();
    let manifest = Document::parse(manifest).unwrap();

    let snforge_std = manifest
        .get("dev-dependencies")
        .unwrap()
        .get("snforge_std")
        .unwrap();
    let snforge_std = snforge_std.as_str().unwrap();
    assert_eq!(snforge_std, env!("CARGO_PKG_VERSION"));
}

#[test]
#[cfg_attr(
    not(feature = "test_for_multiple_scarb_versions"),
    ignore = "Multiple scarb versions must be installed"
)]
fn new_with_minimal_scarb() {
    let temp = tempdir_with_tool_versions().unwrap();
    Command::new("asdf")
        .current_dir(&temp)
        .args(["set", "scarb", "2.12.0"])
        .assert()
        .success();
    runner(&temp)
        .env("DEV_USE_OFFLINE_MODE", "true")
        .args(["new", "abc"])
        .assert()
        .success();

    let manifest = temp.path().join("abc").join("Scarb.toml");
    let manifest = &std::fs::read_to_string(manifest).unwrap();
    let manifest = Document::parse(manifest).unwrap();

    let snforge_std = manifest
        .get("dev-dependencies")
        .unwrap()
        .get("snforge_std")
        .unwrap();
    let snforge_std = snforge_std.as_str().unwrap();
    assert_eq!(snforge_std, env!("CARGO_PKG_VERSION"));
}

#[test]
#[cfg_attr(
    not(feature = "test_for_multiple_scarb_versions"),
    ignore = "Multiple scarb versions must be installed"
)]
fn new_scarb_new_macros() {
    let temp = tempdir_with_tool_versions().unwrap();
    runner(&temp)
        .env("DEV_DISABLE_SNFORGE_STD_DEPENDENCY", "true")
        .args(["new", "abc"])
        .assert()
        .success();
    let snforge_std = Utf8PathBuf::from("../../snforge_std")
        .canonicalize_utf8()
        .unwrap();
    ScarbCommand::new()
        .current_dir(temp.path().join("abc"))
        .args(["add", "snforge_std", "--path", snforge_std.as_str()])
        .command()
        .output()
        .unwrap();

    let output = test_runner(temp.join("abc")).assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
            [..]Compiling[..]
            [..]Finished[..]

            Collected 2 test(s) from abc package
            Running 0 test(s) from src/
            Running 2 test(s) from tests/
            [PASS] abc_integrationtest::test_contract::test_cannot_increase_balance_with_zero_value (l1_gas: ~[..], l1_data_gas: ~[..], l2_gas: ~[..])
            [PASS] abc_integrationtest::test_contract::test_increase_balance (l1_gas: ~[..], l1_data_gas: ~192, l2_gas: ~[..])
            Tests: 2 passed, 0 failed, 0 ignored, 0 filtered out
        "},
    );
}

#[test]
#[cfg_attr(
    not(feature = "test_for_multiple_scarb_versions"),
    ignore = "Multiple scarb versions must be installed"
)]
fn new_scarb_old_macros() {
    let temp = tempdir_with_tool_versions().unwrap();
    runner(&temp)
        .env("DEV_DISABLE_SNFORGE_STD_DEPENDENCY", "true")
        .args(["new", "abc"])
        .assert()
        .success();
    ScarbCommand::new()
        .current_dir(temp.path().join("abc"))
        .args(["add", "snforge_std@0.44.0"])
        .command()
        .output()
        .unwrap();

    let output = test_runner(temp.join("abc")).assert().failure();

    assert_stdout_contains(
        output,
        formatdoc! {r"
            [ERROR] Package snforge_std version does not meet the minimum required version >=0.50.0. Please upgrade snforge_std in Scarb.toml
        ", },
    );
}
