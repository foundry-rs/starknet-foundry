use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use test_utils::running_tests::run_test_case;
use test_utils::{
    runner::{assert_failed, assert_passed},
    test_case,
};

#[test]
fn simple() {
    let test = test_case!(indoc!(
        r"#[test]
        fn simple() {
            assert(2 == 2, '2 == 2');
        }
    "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn failing() {
    let test = test_case!(indoc!(
        r"#[test]
        fn failing() {
            assert(2 == 3, '2 == 3');
        }
    "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
}
