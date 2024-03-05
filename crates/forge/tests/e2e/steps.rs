use crate::e2e::common::runner::setup_package;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

use super::common::runner::test_runner;

#[test]
fn should_allow_less_than_default() {
    let temp = setup_package("steps");

    let output = test_runner(&temp)
        .args(["--max-n-steps", "100000"])
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc!(
            r"
                [..]Compiling[..]
                [..]Finished[..]


                Collected 4 test(s) from steps package
                Running 4 test(s) from src/
                [FAIL] steps::tests::steps_3000017
                
                Failure data:
                    Could not reach the end of the program. RunResources has no remaining steps.
                
                [FAIL] steps::tests::steps_5700031
                
                Failure data:
                    Could not reach the end of the program. RunResources has no remaining steps.
                
                [FAIL] steps::tests::steps_2999998
                
                Failure data:
                    Could not reach the end of the program. RunResources has no remaining steps.
                
                [FAIL] steps::tests::steps_570031
                
                Failure data:
                    Could not reach the end of the program. RunResources has no remaining steps.
                
                Tests: 0 passed, 4 failed, 0 skipped, 0 ignored, 0 filtered out
                
                Failures:
                    steps::tests::steps_3000017
                    steps::tests::steps_5700031
                    steps::tests::steps_2999998
                    steps::tests::steps_570031
            "
        ),
    );
}
#[test]
// 4_000_000 is blockifier limit we want to omit
fn should_allow_more_than_4kk() {
    let temp = setup_package("steps");

    let output = test_runner(&temp)
        .args(["--max-n-steps", "5700031"])
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc!(
            r"
                [..]Compiling[..]
                [..]Finished[..]


                Collected 4 test(s) from steps package
                Running 4 test(s) from src/
                [PASS] steps::tests::steps_570031 [..]
                [PASS] steps::tests::steps_3000017 [..]
                [PASS] steps::tests::steps_2999998 [..]
                [PASS] steps::tests::steps_5700031 [..]
                Tests: 4 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
            "
        ),
    );
}
#[test]
fn should_default_to_3kk() {
    let temp = setup_package("steps");

    let output = test_runner(&temp).assert().code(1);

    assert_stdout_contains(
        output,
        indoc!(
            r"
                [..]Compiling[..]
                [..]Finished[..]


                Collected 4 test(s) from steps package
                Running 4 test(s) from src/
                [PASS] steps::tests::steps_570031 [..]
                [FAIL] steps::tests::steps_3000017
                
                Failure data:
                    Could not reach the end of the program. RunResources has no remaining steps.
                
                [FAIL] steps::tests::steps_5700031
                
                Failure data:
                    Could not reach the end of the program. RunResources has no remaining steps.
                
                [PASS] steps::tests::steps_2999998 [..]
                Tests: 2 passed, 2 failed, 0 skipped, 0 ignored, 0 filtered out
                
                Failures:
                    steps::tests::steps_3000017
                    steps::tests::steps_5700031
            "
        ),
    );
}
