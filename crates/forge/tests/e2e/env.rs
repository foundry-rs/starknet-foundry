use super::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn env_var_reading() {
    let temp = setup_package("env");

    let output = test_runner(&temp)
        .env("FELT_ENV_VAR", "987654321")
        .env("STRING_ENV_VAR", "'abcde'")
        .env(
            "BYTE_ARRAY_ENV_VAR",
            r#""that is a very long environment variable that would normally not fit""#,
        )
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from env package
        Running 1 test(s) from src/
        [PASS] env::tests::reading_env_vars
        Tests: 1 passed, 0 failed, 0 ignored, 0 filtered out
        "},
    );
}
