use crate::helpers::constants::{ACCOUNT_FILE_PATH, SCRIPTS_DIR, URL};
use crate::helpers::fixtures::{copy_script_directory_to_tempdir, get_accounts_path};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use test_case::test_case;

#[test_case("oz_cairo_0"; "cairo_0_account")]
#[test_case("oz_cairo_1"; "cairo_1_account")]
#[test_case("oz"; "oz_account")]
#[test_case("argent"; "argent_account")]
#[test_case("braavos"; "braavos_account")]
#[tokio::test]
async fn test_with_calldata(account: &str) {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/deploy", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "with_calldata";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        account,
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        Success: Script execution completed
        
        Status: success
        "},
    );
}

#[tokio::test]
async fn test_with_fee_settings() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/deploy", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "fee_settings";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user7",
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        Success: Script execution completed
        
        Status: success
        "},
    );
}

#[tokio::test]
async fn test_same_salt_and_class_hash_deployed_twice() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/deploy", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "same_class_hash_and_salt";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user3",
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]
        ScriptCommandError::WaitForTransactionError(WaitForTransactionError::TransactionError(TransactionError::Reverted(ErrorData { msg: "Transaction execution has failed:
        [..]
        [..]: Error in the contract class constructor ([..]):
        Deployment failed: contract already deployed at address [..]
        " })))
        Success: Script execution completed
        
        Status: success
        "#},
    );
}

#[tokio::test]
async fn test_invalid_class_hash() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/deploy", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "invalid_class_hash";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user2",
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]
        ScriptCommandError::WaitForTransactionError(WaitForTransactionError::TransactionError(TransactionError::Reverted(ErrorData { msg: "Transaction execution has failed:
        [..]
        [..]: Error in the contract class constructor ([..]):
        Class with hash [..] is not declared.
        " })))
        Success: Script execution completed
        
        Status: success
        "#},
    );
}

#[tokio::test]
async fn test_invalid_call_data() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/deploy", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "invalid_calldata";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user5",
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]
        ScriptCommandError::WaitForTransactionError(WaitForTransactionError::TransactionError(TransactionError::Reverted(ErrorData { msg: "Transaction execution has failed:
        [..]
        [..]: Error in the contract class constructor ([..]):
        Execution failed. Failure reason:
        Error in contract [..]:
        0x4661696c656420746f20646573657269616c697a6520706172616d202332 ('Failed to deserialize param #2').
        " })))
        Success: Script execution completed
        
        Status: success
        "#},
    );
}

#[tokio::test]
async fn test_invalid_nonce() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/deploy", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "invalid_nonce";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user5",
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {"
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InvalidTransactionNonce(())))
        Success: Script execution completed
        
        Status: success
        "},
    );
}
