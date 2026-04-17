use crate::e2e::common::runner::{setup_package, snforge_test_bin_path, test_runner};
use assert_fs::fixture::{FileWriteStr, PathChild};
use indoc::{formatdoc, indoc};
use shared::test_utils::output_assert::assert_stdout_contains;
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[test]
fn test_fail_wrong_scarb_toml_configuration_for_launch_debugger() {
    let temp = setup_package("debugging");

    let output = test_runner(&temp)
        .arg("test_debugging_trace_success")
        .arg("--exact")
        .arg("--launch-debugger")
        .assert()
        .failure();

    assert_stdout_contains(
        output,
        indoc! {
            "[ERROR] [..]/Scarb.toml must have the following Cairo compiler configuration to launch the debugger:

            [profile.dev.cairo]
            unstable-add-statements-code-locations-debug-info = true
            unstable-add-statements-functions-debug-info = true
            add-functions-debug-info = true
            skip-optimizations = true
            ... other entries ...

            "
        },
    );
}

#[test]
fn test_launch_debugger_waits_for_connection() {
    let temp = setup_package("debugging");

    let manifest_path = temp.child("Scarb.toml");

    let existing = fs::read_to_string(&manifest_path).unwrap();
    manifest_path
        .write_str(&formatdoc!(
            "{existing}
            [profile.dev.cairo]
            unstable-add-statements-code-locations-debug-info = true
            unstable-add-statements-functions-debug-info = true
            add-functions-debug-info = true
            skip-optimizations = true",
        ))
        .unwrap();

    let mut child = Command::new(snforge_test_bin_path())
        .args([
            "test",
            "debugging_integrationtest::test_trace::test_debugging_trace_success",
            "--exact",
            "--launch-debugger",
        ])
        .current_dir(temp.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn snforge process");

    let found_port_line = BufReader::new(child.stdout.take().unwrap())
        .lines()
        .map_while(Result::ok)
        .any(|line| line.contains("DEBUGGER PORT"));

    child.kill().unwrap();
    child.wait().unwrap();

    // For debugging purposes if the test fails.
    let stderr = BufReader::new(child.stderr.take().unwrap())
        .lines()
        .map_while(Result::ok)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        found_port_line,
        "Expected 'DEBUGGER PORT' in snforge output.\n\nstderr:\n{stderr}",
    );
}

#[test]
fn test_launch_debugger_fails_for_fuzzer_test() {
    let temp = setup_package("debugging");

    let manifest_path = temp.child("Scarb.toml");
    let existing = fs::read_to_string(&manifest_path).unwrap();
    manifest_path
        .write_str(&formatdoc!(
            "{existing}
            [profile.dev.cairo]
            unstable-add-statements-code-locations-debug-info = true
            unstable-add-statements-functions-debug-info = true
            add-functions-debug-info = true
            skip-optimizations = true",
        ))
        .unwrap();

    let output = test_runner(&temp)
        .args([
            "debugging_integrationtest::test_trace::test_debugging_fuzzer",
            "--exact",
            "--launch-debugger",
            "--features",
            "fuzzer",
        ])
        .assert()
        .code(2);

    assert_stdout_contains(
        output,
        "[ERROR] --launch-debugger is not supported for fuzzer tests",
    );
}
