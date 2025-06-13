use crate::e2e::common::runner::{runner, test_runner};
use camino::Utf8PathBuf;
use indoc::{formatdoc, indoc};
use scarb_api::ScarbCommand;
use shared::test_utils::output_assert::assert_stdout_contains;
use snapbox::cmd::Command;
use test_utils::tempdir_with_tool_versions;
use toml_edit::ImDocument;

#[test]
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
fn new_with_new_scarb() {
    let temp = tempdir_with_tool_versions().unwrap();
    runner(&temp)
        .env("DEV_USE_OFFLINE_MODE", "true")
        .args(["new", "abc"])
        .assert()
        .success();

    let manifest = temp.path().join("abc").join("Scarb.toml");
    let manifest = &std::fs::read_to_string(manifest).unwrap();
    let manifest = ImDocument::parse(manifest).unwrap();

    let snforge_std = manifest
        .get("dev-dependencies")
        .unwrap()
        .get("snforge_std")
        .unwrap();
    let snforge_std = snforge_std.as_str().unwrap();
    assert_eq!(snforge_std, env!("CARGO_PKG_VERSION"));

    assert!(
        manifest
            .get("dev-dependencies")
            .unwrap()
            .get("snforge_std_deprecated")
            .is_none()
    );
}

#[test]
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
fn new_with_old_scarb() {
    let temp = tempdir_with_tool_versions().unwrap();
    Command::new("asdf")
        .current_dir(&temp)
        .args(["set", "scarb", "2.10.1"])
        .assert()
        .success();
    runner(&temp)
        .env("DEV_USE_OFFLINE_MODE", "true")
        .args(["new", "abc"])
        .assert()
        .success();

    let manifest = temp.path().join("abc").join("Scarb.toml");
    let manifest = &std::fs::read_to_string(manifest).unwrap();
    let manifest = ImDocument::parse(manifest).unwrap();

    let snforge_std = manifest
        .get("dev-dependencies")
        .unwrap()
        .get("snforge_std_deprecated")
        .unwrap();
    let snforge_std = snforge_std.as_str().unwrap();
    assert_eq!(snforge_std, env!("CARGO_PKG_VERSION"));

    assert!(
        manifest
            .get("dev-dependencies")
            .unwrap()
            .get("snforge_std")
            .is_none()
    );
}

#[test]
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
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
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
fn new_scarb_deprecated_macros() {
    let temp = tempdir_with_tool_versions().unwrap();
    runner(&temp)
        .env("DEV_DISABLE_SNFORGE_STD_DEPENDENCY", "true")
        .args(["new", "abc"])
        .assert()
        .success();
    let snforge_std = Utf8PathBuf::from("../../snforge_std_deprecated")
        .canonicalize_utf8()
        .unwrap();
    ScarbCommand::new()
        .current_dir(temp.path().join("abc"))
        .args([
            "add",
            "snforge_std_deprecated",
            "--path",
            snforge_std.as_str(),
        ])
        .command()
        .output()
        .unwrap();
    let scarb_version = ScarbCommand::version()
        .current_dir(temp.path())
        .run()
        .unwrap()
        .scarb
        .to_string();

    let output = test_runner(temp.join("abc")).assert().failure();

    assert_stdout_contains(
        output,
        formatdoc! {r"
            error: the required Cairo version of package snforge_std_deprecated is not compatible with current version
            Cairo version required: <=2.11.4
            Cairo version of Scarb: {scarb_version}

            error: the required Cairo version of each package must match the current Cairo version
            help: pass `--ignore-cairo-version` to ignore Cairo version mismatch
        "},
    );
}

#[test]
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
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

    let output = test_runner(temp.join("abc")).assert().success();
    let current_package_version = env!("CARGO_PKG_VERSION");

    assert_stdout_contains(
        output,
        formatdoc! {r"
            [WARNING] Package snforge_std version does not meet the recommended version requirement ^{current_package_version}, it might result in unexpected behaviour
            [..]Compiling[..]
            [..]Finished[..]

            Collected 2 test(s) from abc package
            Running 0 test(s) from src/
            Running 2 test(s) from tests/
            [PASS] abc_integrationtest::test_contract::test_cannot_increase_balance_with_zero_value (l1_gas: ~[..], l1_data_gas: ~[..], l2_gas: ~[..])
            [PASS] abc_integrationtest::test_contract::test_increase_balance (l1_gas: ~[..], l1_data_gas: ~[..], l2_gas: ~[..])
            Tests: 2 passed, 0 failed, 0 ignored, 0 filtered out
        ", },
    );
}

#[test]
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
fn old_scarb_new_macros() {
    let temp = tempdir_with_tool_versions().unwrap();
    Command::new("asdf")
        .current_dir(&temp)
        .args(["set", "scarb", "2.10.1"])
        .assert()
        .success();
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

    let output = test_runner(temp.join("abc")).assert().failure();

    assert_stdout_contains(
        output,
        indoc! {r"
            [ERROR] On Scarb versions < 2.12.0, the `snforge_std` package must be replaced with `snforge_std_deprecated`. Please update it in Scarb.toml
        "},
    );
}

#[test]
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
fn old_scarb_deprecated_macros() {
    let temp = tempdir_with_tool_versions().unwrap();
    Command::new("asdf")
        .current_dir(&temp)
        .args(["set", "scarb", "2.10.1"])
        .assert()
        .success();
    runner(&temp)
        .env("DEV_DISABLE_SNFORGE_STD_DEPENDENCY", "true")
        .args(["new", "abc"])
        .assert()
        .success();
    let snforge_std = Utf8PathBuf::from("../../snforge_std_deprecated")
        .canonicalize_utf8()
        .unwrap();
    ScarbCommand::new()
        .current_dir(temp.path().join("abc"))
        .args([
            "add",
            "snforge_std_deprecated",
            "--path",
            snforge_std.as_str(),
        ])
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
            [PASS] abc_integrationtest::test_contract::test_increase_balance (l1_gas: ~[..], l1_data_gas: ~[..], l2_gas: ~[..])
            Tests: 2 passed, 0 failed, 0 ignored, 0 filtered out
        "},
    );
}

#[test]
#[cfg_attr(feature = "skip_plugin_checks", ignore = "Plugin checks skipped")]
fn old_scarb_old_macros() {
    let temp = tempdir_with_tool_versions().unwrap();
    Command::new("asdf")
        .current_dir(&temp)
        .args(["set", "scarb", "2.10.1"])
        .assert()
        .success();
    runner(&temp)
        .env("DEV_DISABLE_SNFORGE_STD_DEPENDENCY", "true")
        .args(["new", "abc"])
        .assert()
        .success();
    ScarbCommand::new()
        .current_dir(temp.path().join("abc"))
        .args(["add", "snforge_std", "0.44.0"])
        .command()
        .output()
        .unwrap();

    let output = test_runner(temp.join("abc")).assert().failure();

    assert_stdout_contains(
        output,
        indoc! {r"
            [ERROR] On Scarb versions < 2.12.0, the `snforge_std` package must be replaced with `snforge_std_deprecated`. Please update it in Scarb.toml
        "},
    );
}
