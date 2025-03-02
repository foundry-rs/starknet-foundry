use super::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn features() {
    let temp = setup_package("features");

    let output = test_runner(&temp)
        .arg("--features")
        .arg("snforge_test_only")
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) from features package
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [PASS] features_integrationtest::test::test_mock_function [..]
        [PASS] features_integrationtest::test::test_mock_contract [..]
        Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
        "},
    );
}

#[test]
fn compilation_fails_when_no_features_passed() {
    let temp = setup_package("features");

    let output = test_runner(&temp).assert().failure();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Collected 2 test(s) from features package
        Running 2 test(s) from tests/
        [FAIL] features_integrationtest::test::test_mock_contract

        Failure data:
            "Failed to get contract artifact for name = MockContract."
    "},
    );
}
