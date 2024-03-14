use indoc::indoc;
use test_utils::runner::{assert_case_output_contains, assert_failed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

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
        "Cheatcode `not_existing123` is not supported in this runtime",
    );
}

#[test]
fn cheatcode_invalid_args() {
    let test = test_utils::test_case!(indoc!(
        r"
            use starknet::testing::cheatcode;

            #[test]
            fn cheatcode_invalid_args() {
                cheatcode::<'replace_bytecode'>(array![].span());
                assert(true,'');
            }
        "
    ));

    let result = run_test_case(&test);
    assert_case_output_contains(
        &result,
        "cheatcode_invalid_args",
        indoc!(
            r"
                Got an exception while executing a hint: Hint Error: Reading from buffer failed, this can be caused by calling starknet::testing::cheatcode with invalid arguments.
                Probably snforge_std version is incompatible, check above for incompatibility warning.
            "
        ),
    );
    assert_failed(&result);
}
