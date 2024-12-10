use indoc::indoc;
use test_utils::running_tests::run_test_case;
use test_utils::{
    runner::{assert_case_output_contains, assert_failed},
    test_case,
};

#[test]
fn missing_cheatcode_error() {
    let test = test_case!(indoc!(
        r"
            use starknet::testing::cheatcode;
            use array::ArrayTrait;

            #[test]
            fn missing_cheatcode_error() {
                cheatcode::<'not_existing123'>(array![1, 2].span());
                assert(1==1, 'nothing')
            }
        "
    ));
    let result = run_test_case(&test);
    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "missing_cheatcode_error",
        indoc!(
            r"
            Function `not_existing123` is not supported in this runtime
            Check if used library (`snforge_std` or `sncast_std`) is compatible with used binary, probably one of them is not updated
        "
        ),
    );
}
#[test]
fn cairo_test_cheatcode_error() {
    let test = test_case!(indoc!(
        r"
            use starknet::testing::cheatcode;
            use array::ArrayTrait;

            #[test]
            fn missing_cheatcode_error() {
                cheatcode::<'set_version'>(array![1, 2].span());
                assert(1==1, 'nothing')
            }
        "
    ));
    let result = run_test_case(&test);
    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "missing_cheatcode_error",
        indoc!(
            r"
            Function `set_version` is not supported in this runtime
            Check if functions are imported from `snforge_std`/`sncast_std` NOT from `starknet::testing`
        "
        ),
    );
}
