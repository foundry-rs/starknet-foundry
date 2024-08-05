use crate::helpers::constants::ACCOUNT_FILE_PATH;
use crate::helpers::runner::runner;
use crate::{e2e::account::helpers::create_tempdir_with_accounts_file, helpers::constants::URL};
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, AsOutput};

#[test]
pub fn test_no_accounts_in_network() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "account",
        "delete",
        "--url",
        URL,
        "--name",
        "user99",
        "--network",
        "my-custom-network",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account delete
        error: No accounts defined for network = my-custom-network
        "},
    );
}

#[test]
pub fn test_account_does_not_exist() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "account",
        "delete",
        "--url",
        URL,
        "--name",
        "user99",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account delete
        error: Account with name user99 does not exist
        "},
    );
}

#[test]
pub fn test_delete_abort() {
    // Creating dummy accounts test file
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);

    // Now delete dummy account
    let args = vec![
        "--accounts-file",
        &accounts_file_name,
        "account",
        "delete",
        "--url",
        URL,
        "--name",
        "user3",
        "--network",
        "custom-network",
    ];

    // Run test with a negative user input
    let snapbox = runner(&args).current_dir(temp_dir.path()).stdin("n");

    let output = snapbox.assert().success();
    assert_stderr_contains(
        output,
        indoc! {r"
        command: account delete
        error: Delete aborted
        "},
    );
}

#[test]
pub fn test_happy_case() {
    // Creating dummy accounts test file
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);

    // Now delete dummy account
    let args = vec![
        "--accounts-file",
        &accounts_file_name,
        "account",
        "delete",
        "--url",
        URL,
        "--name",
        "user3",
        "--network",
        "custom-network",
    ];

    // Run test with an affirmative user input
    let snapbox = runner(&args).current_dir(temp_dir.path()).stdin("Y");

    snapbox.assert().success().stdout_matches(indoc! {r"
        command: account delete
        result: Account successfully removed
    "});
}

#[test]
pub fn test_happy_case_without_network_args() {
    // Creating dummy accounts test file
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);

    // Now delete dummy account
    let args = vec![
        "--accounts-file",
        &accounts_file_name,
        "account",
        "delete",
        "--url",
        URL,
        "--name",
        "user0",
    ];

    // Run test with an affirmative user input
    let snapbox = runner(&args).current_dir(temp_dir.path()).stdin("Y");

    snapbox.assert().success().stdout_matches(indoc! {r"
        command: account delete
        result: Account successfully removed
    "});
}

#[test]
pub fn test_happy_case_with_yes_flag() {
    // Creating dummy accounts test file
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);

    // Now delete dummy account
    let args = vec![
        "--accounts-file",
        &accounts_file_name,
        "account",
        "delete",
        "--url",
        URL,
        "--name",
        "user3",
        "--network",
        "custom-network",
        "--yes",
    ];

    // Run test with no additional user input
    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stderr().is_empty());
    output.stdout_matches(indoc! {r"
        command: account delete
        result: Account successfully removed
    "});
}
