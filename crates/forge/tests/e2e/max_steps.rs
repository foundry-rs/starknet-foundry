use crate::assert_stdout_contains;
use crate::e2e::common::runner::{setup_package, test_runner};
use indoc::indoc;

#[test]
fn max_steps_flag() {
    let temp = setup_package("max_steps");
    let snapbox = test_runner();

    let output = snapbox
        .current_dir(&temp)
        .arg("-m")
        .arg("10")
        .assert()
        .code(1);
    assert_stdout_contains!(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) from max_steps package
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [FAIL] tests::ext_function_test::simple_test

        Failure data: Max steps limit exceeded. Limit: 10. Executed: 117
        [FAIL] tests::ext_function_test::hard_test

        Failure data: Max steps limit exceeded. Limit: 10. Executed: 758639
        Tests: 0 passed, 2 failed, 0 skipped, 0 ignored, 0 filtered out

        Failures:
            tests::ext_function_test::simple_test
            tests::ext_function_test::hard_test
        "}
    );
}
