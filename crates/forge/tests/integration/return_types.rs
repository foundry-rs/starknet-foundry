//! A test function may declare any return type, including one that does not implement `Default`.
//! `NonZero` has no meaningful default, so `Result<NonZero<u8>, u8>` stands in for such a type.

use crate::utils::running_tests::run_test_case;
use crate::utils::{runner::assert_passed, test_case};
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

#[test]
fn return_type_without_default_with_config_attribute() {
    let test = test_case!(indoc!(
        r"#[test]
        #[available_gas(l2_gas: 40000000)]
        fn return_type_without_default_with_config_attribute() -> Result<NonZero<u8>, u8> {
            Result::Ok(5)
        }
    "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
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
