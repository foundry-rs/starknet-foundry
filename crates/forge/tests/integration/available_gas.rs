use indoc::indoc;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_case_output_contains, assert_failed, assert_passed};

#[test]
fn correct_available_gas() {
    let test = test_utils::test_case!(indoc!(
        r"
            #[test]
            #[available_gas(21)]
            fn keccak_cost() {
                keccak::keccak_u256s_le_inputs(array![1].span());
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn available_gas_exceeded() {
    let test = test_utils::test_case!(indoc!(
        r"
            #[test]
            #[available_gas(20)]
            fn keccak_cost() {
                keccak::keccak_u256s_le_inputs(array![1].span());
            }
        "
    ));

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(result, "keccak_cost", "available_gas exceeded");
}
