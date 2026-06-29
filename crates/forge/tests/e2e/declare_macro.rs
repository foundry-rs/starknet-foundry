use crate::e2e::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn declare_macro() {
    let temp = setup_package("declare_macro");

    let output = test_runner(&temp).assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 5 test(s) from declare_macro package
        Running 0 test(s) from src/
        Running 5 test(s) from tests/
        [PASS] declare_macro_integrationtest::tests::declare_with_full_path [..]
        [PASS] declare_macro_integrationtest::tests::declare_with_partial_path [..]
        [PASS] declare_macro_integrationtest::tests::declare_with_module_alias_is_not_resolved_as_canonical_path [..]
        [PASS] declare_macro_integrationtest::tests::declare_with_contract_name [..]
        [PASS] declare_macro_integrationtest::tests::declare_with_contract_alias_is_not_resolved_as_canonical_path [..]
        Tests: 5 passed, 0 failed, 0 ignored, 0 filtered out
        "},
    );
}
