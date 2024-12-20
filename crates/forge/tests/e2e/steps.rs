use super::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

// TODO(#2806)

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
                [FAIL] steps::tests::steps_more_than_10000000

                Failure data:
                    Could not reach the end of the program. RunResources has no remaining steps.

                [FAIL] steps::tests::steps_less_than_10000000

                Failure data:
                    Could not reach the end of the program. RunResources has no remaining steps.

                [FAIL] steps::tests::steps_much_more_than_10000000

                Failure data:
                    Could not reach the end of the program. RunResources has no remaining steps.

                [FAIL] steps::tests::steps_much_less_than_10000000

                Failure data:
                    Could not reach the end of the program. RunResources has no remaining steps.

                Tests: 0 passed, 4 failed, 0 skipped, 0 ignored, 0 filtered out

                Failures:
                    steps::tests::steps_more_than_10000000
                    steps::tests::steps_less_than_10000000
                    steps::tests::steps_much_more_than_10000000
                    steps::tests::steps_much_less_than_10000000
            "
        ),
    );
}
#[test]
// 10_000_000 is blockifier limit we want to omit
fn should_allow_more_than_10m() {
    let temp = setup_package("steps");

    let output = test_runner(&temp)
        .args(["--max-n-steps", "15000100"])
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
                [PASS] steps::tests::steps_much_less_than_10000000 [..]
                [PASS] steps::tests::steps_more_than_10000000 [..]
                [PASS] steps::tests::steps_less_than_10000000 [..]
                [PASS] steps::tests::steps_much_more_than_10000000 [..]
                Tests: 4 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
            "
        ),
    );
}
#[test]
fn should_default_to_10m() {
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
            [PASS] steps::tests::steps_much_less_than_10000000 [..]
            [FAIL] steps::tests::steps_much_more_than_10000000

            Failure data:
                Could not reach the end of the program. RunResources has no remaining steps.

            [FAIL] steps::tests::steps_more_than_10000000

            Failure data:
                Could not reach the end of the program. RunResources has no remaining steps.

            [PASS] steps::tests::steps_less_than_10000000 [..]
            Tests: 2 passed, 2 failed, 0 skipped, 0 ignored, 0 filtered out

            Failures:
                steps::tests::steps_much_more_than_10000000
                steps::tests::steps_more_than_10000000
            "
        ),
    );
}
