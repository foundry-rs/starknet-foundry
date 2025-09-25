use super::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn release() {
    let temp = setup_package("empty");

    let output = test_runner(&temp).arg("--release").assert().code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished `release` profile target(s) in [..]


        Collected 0 test(s) from empty package
        Tests: 0 passed, 0 failed, 0 ignored, 0 filtered out
        "},
    );
}

#[test]
fn custom() {
    let temp = setup_package("empty");

    let output = test_runner(&temp)
        .args(["--profile", "custom-profile"])
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished `custom-profile` profile target(s) in [..]


        Collected 0 test(s) from empty package
        Tests: 0 passed, 0 failed, 0 ignored, 0 filtered out
        "},
    );
}
