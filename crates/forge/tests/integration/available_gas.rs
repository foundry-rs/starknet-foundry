use crate::utils::runner::{assert_available_gas_exceeded, assert_failed, assert_passed};
use crate::utils::running_tests::run_test_case;
use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use starknet_api::execution_resources::{GasAmount, GasVector};

#[test]
fn correct_available_gas() {
    let test = crate::utils::test_case!(indoc!(
        r"
            #[test]
            #[available_gas(l2_gas: 440000)]
            fn keccak_cost() {
                keccak::keccak_u256s_le_inputs(array![1].span());
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn available_gas_exceeded() {
    let test = crate::utils::test_case!(indoc!(
        r"
            #[test]
            #[available_gas(l2_gas: 5)]
            fn keccak_cost() {
                keccak::keccak_u256s_le_inputs(array![1].span());
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
    assert_available_gas_exceeded(
        &result,
        "keccak_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(888),
        },
    );
}

#[test]
fn available_gas_fuzzing() {
    let test = crate::utils::test_case!(indoc!(
        r"
            #[test]
            #[available_gas(l2_gas: 40000000)]
            #[fuzzer]
            fn keccak_cost(x: u256) {
                keccak::keccak_u256s_le_inputs(array![x].span());
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}
