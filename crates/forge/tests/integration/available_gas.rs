use crate::utils::runner::{assert_case_output_contains, assert_failed, assert_passed};
use crate::utils::running_tests::run_test_case;
use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;

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
fn correct_corelib_available_gas() {
    let test = crate::utils::test_case!(indoc!(
        r"
            #[test]
            #[available_gas(30000)]
            fn simple() {
                assert(2 == 2, '2 == 2');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

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
    assert_case_output_contains(
        &result,
        "keccak_cost",
        "Test cost exceeded the available gas. Consumed l1_gas: ~0, l1_data_gas: ~0, l2_gas: ~240000",
    );
}

#[test]
fn corelib_available_gas_exceeded() {
    let test = crate::utils::test_case!(indoc!(
        r"
            #[test]
            #[available_gas(1)]
            fn simple() {
                assert(2 == 2, '2 == 2');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "simple",
        "Test cost exceeded the available sierra gas. Consumed sierra_gas: ~",
    );
}

#[test]
fn named_sierra_gas_exceeded() {
    let test = crate::utils::test_case!(indoc!(
        r"
            #[test]
            #[available_gas(sierra_gas: 1)]
            fn simple() {
                assert(2 == 2, '2 == 2');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "simple",
        "Test cost exceeded the available sierra gas. Consumed sierra_gas: ~",
    );
}

#[test]
fn sierra_gas_with_cairo_steps_tracking_fails() {
    let test = crate::utils::test_case!(indoc!(
        r"
            #[test]
            #[available_gas(100)]
            fn simple() {
                assert(2 == 2, '2 == 2');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "simple",
        "Setting a Sierra gas limit via `#[available_gas]` requires running the test with Sierra gas tracking, but it is run with Cairo steps tracking. Use resource bounds (`l1_gas`, `l1_data_gas`, `l2_gas`) instead, or run with Sierra gas tracking.",
    );
}

#[test]
fn named_sierra_gas_with_cairo_steps_tracking_fails() {
    let test = crate::utils::test_case!(indoc!(
        r"
            #[test]
            #[available_gas(sierra_gas: 100)]
            fn simple() {
                assert(2 == 2, '2 == 2');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "simple",
        "Setting a Sierra gas limit via `#[available_gas]` requires running the test with Sierra gas tracking, but it is run with Cairo steps tracking. Use resource bounds (`l1_gas`, `l1_data_gas`, `l2_gas`) instead, or run with Sierra gas tracking.",
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
