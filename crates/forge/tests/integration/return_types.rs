use crate::utils::running_tests::run_test_case;
use crate::utils::{
    runner::{assert_case_output_contains, assert_failed, assert_passed},
    test_case,
};
use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;

#[test]
fn return_type_without_default() {
    let test = test_case!(indoc!(
        r"#[test]
        fn return_type_without_default() -> Result<NonZero<u8>, u8> {
            assert(2 == 2, '2 == 2');
            Result::Ok(5)
        }
    "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

// The only test here sensitive to config delivery. If the config never leaves the test function the
// gas limit is not applied and the test passes, so this one has to assert failure to have teeth.
#[test]
fn return_type_without_default_with_available_gas_exceeded() {
    let test = test_case!(indoc!(
        r"#[test]
        #[available_gas(l2_gas: 5)]
        fn return_type_without_default_with_available_gas_exceeded() -> Result<NonZero<u8>, u8> {
            keccak::keccak_u256s_le_inputs(array![1].span());
            Result::Ok(5)
        }
    "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "return_type_without_default_with_available_gas_exceeded",
        "Test cost exceeded the available gas",
    );
}

#[test]
fn return_type_without_default_with_fuzzer() {
    let test = test_case!(indoc!(
        r"#[test]
        #[fuzzer]
        fn return_type_without_default_with_fuzzer(a: u8) -> Result<NonZero<u8>, u8> {
            Result::Ok(1)
        }
    "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

// `#[fuzzer]` expanding before `#[test]` makes the test wrapper delegate the config run through the
// generated fuzzer function, which is a different shape than the opposite order produces.
#[test]
fn return_type_without_default_with_fuzzer_before_test() {
    let test = test_case!(indoc!(
        r"#[fuzzer]
        #[test]
        fn return_type_without_default_with_fuzzer_before_test(a: u8) -> Result<NonZero<u8>, u8> {
            Result::Ok(1)
        }
    "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn return_type_without_default_with_test_case() {
    let test = test_case!(indoc!(
        r"#[test]
        #[test_case(1)]
        #[test_case(2)]
        fn return_type_without_default_with_test_case(a: u8) -> Result<NonZero<u8>, u8> {
            Result::Ok(1)
        }
    "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

// With both `#[fuzzer]` and `#[test_case]` present, only one of them may attach the marker that
// lifts the config out - two markers would generate the config function twice.
#[test]
fn return_type_without_default_with_fuzzer_and_test_case() {
    let test = test_case!(indoc!(
        r#"#[test]
        #[fuzzer]
        #[test_case(name: "explicit", 1)]
        fn return_type_without_default_with_fuzzer_and_test_case(a: u8) -> Result<NonZero<u8>, u8> {
            Result::Ok(1)
        }
    "#
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}
