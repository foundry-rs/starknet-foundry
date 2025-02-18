use indoc::indoc;
use test_utils::runner::{assert_case_output_contains, assert_failed, assert_passed};
use test_utils::running_tests::run_test_case;

#[test]
fn correct_available_gas() {
    let test = test_utils::test_case!(indoc!(
        r"
            #[test]
            #[available_gas(11)]
            fn keccak_cost() {
                keccak::keccak_u256s_le_inputs(array![1].span());
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn available_gas_exceeded() {
    let test = test_utils::test_case!(indoc!(
        r"
            #[test]
            #[available_gas(5)]
            fn keccak_cost() {
                keccak::keccak_u256s_le_inputs(array![1].span());
            }
        "
    ));

    let result = run_test_case(&test);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "keccak_cost",
        "Test cost exceeded the available gas. Consumed gas: ~6",
    );
}

#[test]
fn available_gas_fuzzing() {
    let test = test_utils::test_case!(indoc!(
        r"
            #[test]
            #[available_gas(100)]
            #[fuzzer]
            fn keccak_cost(x: u256) {
                keccak::keccak_u256s_le_inputs(array![x].span());
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed(&result);
}
