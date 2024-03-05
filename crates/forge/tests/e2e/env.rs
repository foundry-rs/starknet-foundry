use crate::e2e::common::runner::setup_package;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

use super::common::runner::test_runner;

#[test]
fn env_var_reading() {
    let temp = setup_package("env");

    let output = test_runner(&temp)
        .env("FELT_ENV_VAR", "987654321")
        .env("STRING_ENV_VAR", "'abcde'")
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from env package
        Running 1 test(s) from src/
        [PASS] env::tests::reading_env_vars [..]
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
        "},
    );
}
