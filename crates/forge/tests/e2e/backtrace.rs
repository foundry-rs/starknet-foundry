use super::common::runner::{setup_package, test_runner};
use crate::assert_cleaned_output;
use assert_fs::TempDir;
use assert_fs::fixture::{FileWriteStr, PathChild};
use indoc::indoc;
use shared::test_utils::output_assert::{AsOutput, assert_stdout_contains};
use std::fs;
use toml_edit::{DocumentMut, value};

#[test]
fn test_backtrace_missing_env() {
    let temp = setup_package("backtrace_vm_error");

    let output = test_runner(&temp).assert().failure();

    assert_stdout_contains(
        output,
        indoc! {
           "Failure data:
            Got an exception while executing a hint: Requested contract address 0x0000000000000000000000000000000000000000000000000000000000000123 is not deployed.
            note: run with `SNFORGE_BACKTRACE=1` environment variable to display a backtrace"
        },
    );
}

#[cfg_attr(not(feature = "cairo-native"), ignore)]
#[test]
fn test_backtrace_native_execution() {
    let temp = setup_package("backtrace_vm_error");

    let output = test_runner(&temp)
        .arg("--run-native")
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .code(2);

    assert_stdout_contains(
        output,
        "[ERROR] Backtrace generation is not supported with `cairo-native` execution\n",
    );
}

#[test]
fn test_backtrace() {
    let temp = setup_package("backtrace_vm_error");

    let output = test_runner(&temp)
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .failure();

    assert_cleaned_output!(output);
}

#[test]
fn test_backtrace_without_inlines() {
    let temp = setup_package("backtrace_vm_error");
    without_inlines(&temp);

    let output = test_runner(&temp)
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .failure();

    assert_cleaned_output!(output);
}

#[test]
fn test_wrong_scarb_toml_configuration() {
    let temp = setup_package("backtrace_vm_error");

    let manifest_path = temp.child("Scarb.toml");

    let mut scarb_toml = fs::read_to_string(&manifest_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();

    scarb_toml["profile"]["dev"]["cairo"]["unstable-add-statements-code-locations-debug-info"] =
        value(false);

    manifest_path.write_str(&scarb_toml.to_string()).unwrap();

    let output = test_runner(&temp)
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .failure();

    assert_stdout_contains(
        output,
        indoc! {
           "[ERROR] Scarb.toml must have the following Cairo compiler configuration to run backtrace:

            [profile.dev.cairo]
            unstable-add-statements-functions-debug-info = true
            unstable-add-statements-code-locations-debug-info = true
            panic-backtrace = true
            ... other entries ..."
        },
    );
}

#[test]
fn test_backtrace_panic() {
    let temp = setup_package("backtrace_panic");

    let output = test_runner(&temp)
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .failure();

    assert_cleaned_output!(output);
}

#[test]
fn test_backtrace_panic_without_optimizations() {
    let temp = setup_package("backtrace_panic");

    let manifest_path = temp.child("Scarb.toml");

    let mut scarb_toml = fs::read_to_string(&manifest_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();

    scarb_toml["cairo"]["skip-optimizations"] = value(true);
    manifest_path.write_str(&scarb_toml.to_string()).unwrap();

    let output = test_runner(&temp)
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .failure();

    assert_cleaned_output!(output);
}

#[test]
fn test_backtrace_panic_without_inlines() {
    let temp = setup_package("backtrace_panic");
    without_inlines(&temp);

    let output = test_runner(&temp)
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .failure();

    assert_cleaned_output!(output);
}

#[test]
fn test_handled_error_not_display() {
    let temp = setup_package("dispatchers");

    let output = test_runner(&temp)
        .arg("test_handle_and_panic")
        .env("SNFORGE_BACKTRACE", "1")
        .assert()
        .success();

    // Error from the `FailableContract` should not appear in the output
    assert!(
        !output
            .as_stdout()
            .contains("error occurred in contract 'FailableContract'")
    );

    assert_cleaned_output!(output);
}

fn without_inlines(temp_dir: &TempDir) {
    let manifest_path = temp_dir.child("Scarb.toml");

    let mut scarb_toml = fs::read_to_string(&manifest_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();

    scarb_toml["profile"]["dev"]["cairo"]["inlining-strategy"] = value("avoid");

    manifest_path.write_str(&scarb_toml.to_string()).unwrap();
}
