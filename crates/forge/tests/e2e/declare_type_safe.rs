use crate::e2e::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn declare_type_safe() {
    let temp = setup_package("declare_type_safe");

    let output = test_runner(&temp).assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 3 test(s) from declare_type_safe package
        Running 3 test(s) from tests/
        [PASS] declare_type_safe_integrationtest::contract::declare_contract_name [..]
        [PASS] declare_type_safe_integrationtest::contract::declare_with_full_module_path [..]
        [PASS] declare_type_safe_integrationtest::contract::declare_with_partial_module_path [..]
        Running 0 test(s) from src/
        Tests: 3 passed, 0 failed, 0 ignored, 0 filtered out
        "},
    );
}
