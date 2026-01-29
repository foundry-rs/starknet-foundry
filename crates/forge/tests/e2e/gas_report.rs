use crate::assert_cleaned_output;
use crate::e2e::common::runner::{
    BASE_FILE_PATTERNS, Package, setup_package, setup_package_with_file_patterns, test_runner,
};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn snap_basic() {
    let temp = setup_package("simple_package");
    let output = test_runner(&temp)
        .arg("--gas-report")
        .arg("call_and_invoke")
        .assert()
        .code(0);

    assert_cleaned_output!(output);
}

#[test]
fn snap_recursive_calls() {
    let temp = setup_package("debugging");
    let output = test_runner(&temp)
        .arg("--gas-report")
        .arg("test_debugging_trace_success")
        .assert()
        .code(0);

    assert_cleaned_output!(output);
}

#[test]
fn snap_multiple_contracts_and_constructor() {
    let temp = setup_package("simple_package_with_cheats");
    let output = test_runner(&temp)
        .arg("--gas-report")
        .arg("call_and_invoke_proxy")
        .assert()
        .code(0);

    assert_cleaned_output!(output);
}

#[test]
fn snap_fork() {
    let temp =
        setup_package_with_file_patterns(Package::Name("forking".to_string()), BASE_FILE_PATTERNS);

    let output = test_runner(&temp)
        .arg("--gas-report")
        .arg("test_track_resources")
        .assert()
        .code(0);

    assert_cleaned_output!(output);
}

#[test]
fn no_transactions() {
    let temp = setup_package("simple_package");
    let output = test_runner(&temp)
        .arg("--gas-report")
        .arg("test_fib")
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
    [..]Compiling[..]
    [..]Finished[..]


    Collected 1 test(s) from simple_package package
    Running 1 test(s) from src/
    [PASS] simple_package::tests::test_fib (l1_gas: ~0, l1_data_gas: ~0, l2_gas: ~[..])
    No contract gas usage data to display, no contract calls made.

    Running 0 test(s) from tests/
    Tests: 1 passed, 0 failed, 0 ignored, [..] filtered out
    "},
    );
}
