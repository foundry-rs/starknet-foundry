use indoc::indoc;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_case_output_contains, assert_failed, test_case};

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
    assert_failed!(result);
    assert_case_output_contains!(
        result,
        "missing_cheatcode_error",
        "Cheatcode `not_existing123` is not supported in this runtime"
    );
}
