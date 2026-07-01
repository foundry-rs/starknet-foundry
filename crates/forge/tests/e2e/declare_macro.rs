use crate::e2e::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn with_full_path() {
    let temp = setup_package("declare_macro");

    let output = test_runner(&temp)
        .arg("declare_with_full_path")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Collected 1 test(s) from declare_macro package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] declare_macro_integrationtest::tests::declare_with_full_path [..]
        Tests: 1 passed, 0 failed, 0 ignored, 4 filtered out
        "},
    );
}

#[test]
fn with_partial_path() {
    let temp = setup_package("declare_macro");

    let output = test_runner(&temp)
        .arg("declare_with_partial_path")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Collected 1 test(s) from declare_macro package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] declare_macro_integrationtest::tests::declare_with_partial_path [..]
        Tests: 1 passed, 0 failed, 0 ignored, 4 filtered out
        "},
    );
}

#[test]
fn with_contract_name() {
    let temp = setup_package("declare_macro");

    let output = test_runner(&temp)
        .arg("declare_with_contract_name")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Collected 1 test(s) from declare_macro package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] declare_macro_integrationtest::tests::declare_with_contract_name [..]
        Tests: 1 passed, 0 failed, 0 ignored, 4 filtered out
        "},
    );
}

#[test]
fn with_module_alias_is_not_resolved_as_canonical_path() {
    let temp = setup_package("declare_macro");

    let output = test_runner(&temp)
        .arg("declare_with_module_alias_is_not_resolved_as_canonical_path")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Collected 1 test(s) from declare_macro package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] declare_macro_integrationtest::tests::declare_with_module_alias_is_not_resolved_as_canonical_path [..]
        Tests: 1 passed, 0 failed, 0 ignored, 4 filtered out
        "},
    );
}

#[test]
fn with_contract_alias_is_not_resolved_as_canonical_path() {
    let temp = setup_package("declare_macro");

    let output = test_runner(&temp)
        .arg("declare_with_contract_alias_is_not_resolved_as_canonical_path")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Collected 1 test(s) from declare_macro package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] declare_macro_integrationtest::tests::declare_with_contract_alias_is_not_resolved_as_canonical_path [..]
        Tests: 1 passed, 0 failed, 0 ignored, 4 filtered out
        "},
    );
}
