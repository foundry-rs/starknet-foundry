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

                Collected 3 test(s) from steps package
                Running 3 test(s) from src/
                [FAIL] steps::tests::steps_less_than_10_000_000

                Failure data:
                    Could not reach the end of the program. RunResources has no remaining steps.

                [FAIL] steps::tests::steps_more_than_10_000_000

                Failure data:
                    Could not reach the end of the program. RunResources has no remaining steps.

                [FAIL] steps::tests::steps_more_than_100_000_000
                
                Failure data:
                    Could not reach the end of the program. RunResources has no remaining steps.

                Tests: 0 passed, 3 failed, 0 ignored, 0 filtered out

                Failures:
                    steps::tests::steps_less_than_10_000_000
                    steps::tests::steps_more_than_10_000_000
                    steps::tests::steps_more_than_100_000_000
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
        .code(1);

    assert_stdout_contains(
        output,
        indoc!(
            r"
                [..]Compiling[..]
                [..]Finished[..]

                Collected 3 test(s) from steps package
                Running 3 test(s) from src/
                [PASS] steps::tests::steps_more_than_10_000_000 [..]
                [PASS] steps::tests::steps_less_than_10_000_000 [..]
                [FAIL] steps::tests::steps_more_than_100_000_000

                Failure data:
                    Could not reach the end of the program. RunResources has no remaining steps.

                Tests: 2 passed, 1 failed, 0 ignored, 0 filtered out

                Failures:
                    steps::tests::steps_more_than_100_000_000
            "
        ),
    );
}

#[test]
fn should_default_to_usize_max() {
    let temp = setup_package("steps");

    let output = test_runner(&temp).assert().code(0);

    assert_stdout_contains(
        output,
        indoc!(
            r"
            [..]Compiling[..]
            [..]Finished[..]

            Collected 3 test(s) from steps package
            Running 3 test(s) from src/
            [PASS] steps::tests::steps_less_than_10_000_000 (l1_gas: ~[..], l1_data_gas: ~[..], l2_gas: ~[..])
            [PASS] steps::tests::steps_more_than_10_000_000 (l1_gas: ~[..], l1_data_gas: ~[..], l2_gas: ~[..])
            [PASS] steps::tests::steps_more_than_100_000_000 (l1_gas: ~[..], l1_data_gas: ~[..], l2_gas: ~[..])

            Tests: 3 passed, 0 failed, 0 ignored, 0 filtered out
            "
        ),
    );
}
