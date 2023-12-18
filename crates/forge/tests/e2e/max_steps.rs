use crate::assert_stdout_contains;
use crate::e2e::common::runner::{setup_package, test_runner};
use indoc::indoc;

#[test]
fn max_steps_flag() {
    let temp = setup_package("max_steps");
    let snapbox = test_runner();

    let output = snapbox
        .current_dir(&temp)
        .arg("without_attr")
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
        [FAIL] tests::without_attr::simple_test

        Failure data: Max steps limit exceeded. Limit: 10. Executed: 117
        [FAIL] tests::without_attr::hard_test

        Failure data: Max steps limit exceeded. Limit: 10. Executed: 758639
        Tests: 0 passed, 2 failed, 0 skipped, 0 ignored, 2 filtered out

        Failures:
            tests::without_attr::simple_test
            tests::without_attr::hard_test
        "}
    );
}

#[test]
fn max_steps_attr() {
    let temp = setup_package("max_steps");
    let snapbox = test_runner();

    let output = snapbox.current_dir(&temp).arg("with_attr").assert().code(1);
    assert_stdout_contains!(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) from max_steps package
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [PASS] tests::with_attr::simple_test, gas: ~2
        [FAIL] tests::with_attr::hard_test
        
        Failure data: Max steps limit exceeded. Limit: 120. Executed: 758639
        Tests: 1 passed, 1 failed, 0 skipped, 0 ignored, 2 filtered out
        
        Failures:
            tests::with_attr::hard_test
        "}
    );
}
