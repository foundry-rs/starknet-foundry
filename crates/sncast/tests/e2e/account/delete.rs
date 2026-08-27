use crate::helpers::constants::ACCOUNT_FILE_PATH;
use crate::helpers::runner::runner;
use crate::{e2e::account::helpers::create_tempdir_with_accounts_file, helpers::constants::URL};
use indoc::indoc;
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains};
use test_case::test_case;

#[test]
pub fn test_no_accounts_in_network() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "account",
        "delete",
        "--name",
        "user99",
        "--network-name",
        "my-custom-network",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account delete
        Error: No accounts defined for network = my-custom-network
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
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account delete
        Error: Account with name user99 does not exist
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
        "--name",
        "user3",
        "--network-name",
        "custom-network",
        "--yes",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());

    snapbox.assert().success().stdout_eq(indoc! {r"
        [WARNING] Accounts file was migrated to schema version 2; the original was saved as a .v1.bak file
        Success: Account deleted

        Account successfully removed
    "});
}

#[test]
pub fn test_happy_case_url() {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);

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

    let snapbox = runner(&args).current_dir(temp_dir.path()).stdin("Y");

    snapbox.assert().success().stdout_eq(indoc! {r"
        [WARNING] Accounts file was migrated to schema version 2; the original was saved as a .v1.bak file
        Success: Account deleted

        Account successfully removed
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
        "--name",
        "user3",
        "--network-name",
        "custom-network",
        "--yes",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());

    snapbox.assert().success().stdout_eq(indoc! {r"
        [WARNING] Accounts file was migrated to schema version 2; the original was saved as a .v1.bak file
        Success: Account deleted

        Account successfully removed
    "});
}

#[test]
pub fn test_accept_only_one_network_type_argument() {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);

    let args = vec![
        "--accounts-file",
        &accounts_file_name,
        "account",
        "delete",
        "--url",
        URL,
        "--name",
        "user3",
        "--network-name",
        "custom-network",
        "--yes",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());

    let output = snapbox.assert().failure();
    assert_stderr_contains(
        output,
        indoc! {r"
            error: the argument '--url <URL>' cannot be used with '--network-name <NETWORK_NAME>'
        "},
    );
}

#[test]
fn test_empty_input_defaults_to_no() {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);

    // Now delete dummy account
    let args = vec![
        "--accounts-file",
        &accounts_file_name,
        "account",
        "delete",
        "--name",
        "user3",
        "--network-name",
        "custom-network",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path()).stdin("\n");

    let output = snapbox.assert().failure();
    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account delete
        Error: Delete aborted
        "},
    );
}

#[test_case("Y")]
#[test_case("y")]
#[test_case("Yes")]
#[test_case("yes")]
fn test_yes_to_prompt(input: &str) {
    // Creating dummy accounts test file
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);

    // Now delete dummy account
    let args = vec![
        "--accounts-file",
        &accounts_file_name,
        "account",
        "delete",
        "--name",
        "user3",
        "--network-name",
        "custom-network",
    ];

    // Run test with an affirmative user input
    let snapbox = runner(&args)
        .current_dir(temp_dir.path())
        .stdin(format!("{input}\n"));

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Account deleted

        Account successfully removed
    "});
}

#[test_case("N")]
#[test_case("n")]
#[test_case("No")]
#[test_case("no")]
fn test_no_to_prompt(input: &str) {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);

    // Now delete dummy account
    let args = vec![
        "--accounts-file",
        &accounts_file_name,
        "account",
        "delete",
        "--name",
        "user3",
        "--network-name",
        "custom-network",
    ];

    // Run test with a negative user input
    let snapbox = runner(&args)
        .current_dir(temp_dir.path())
        .stdin(format!("{input}\n"));

    let output = snapbox.assert().failure();
    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account delete
        Error: Delete aborted
        "},
    );
}

#[test_case("yay")]
#[test_case("nay")]
#[test_case("maybe")]
fn test_reprompt(input: &str) {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true);

    // Now delete dummy account
    let args = vec![
        "--accounts-file",
        &accounts_file_name,
        "account",
        "delete",
        "--name",
        "user3",
        "--network-name",
        "custom-network",
    ];

    let snapbox = runner(&args)
        .current_dir(temp_dir.path())
        .stdin(format!("{input}\n\n"));

    let output = snapbox.assert().failure();
    // Verify reprompt
    assert_stderr_contains(
        output.as_stderr().to_string(),
        format!("Could not parse {input} as bool."),
    );
    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account delete
        Error: Delete aborted
        "},
    );
}
