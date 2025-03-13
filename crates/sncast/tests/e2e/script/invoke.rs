use crate::helpers::constants::{ACCOUNT_FILE_PATH, SCRIPTS_DIR, URL};
use crate::helpers::fixtures::{copy_script_directory_to_tempdir, get_accounts_path};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use test_case::test_case;

#[test_case("oz_cairo_0"; "cairo_0_account")]
#[test_case("oz_cairo_1"; "cairo_1_account")]
#[test_case("oz"; "oz_account")]
// TODO(#3089)
// #[test_case("argent"; "argent_account")]
// #[test_case("braavos"; "braavos_account")]
#[tokio::test]
async fn test_insufficient_resource_for_validate(account: &str) {
    let script_dir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/invoke", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "max_fee_too_low";
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

    let snapbox = runner(&args).current_dir(script_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InsufficientResourcesForValidate(())))
        command: script run
        status: success
        "},
    );
}

#[tokio::test]
async fn test_contract_does_not_exist() {
    let script_dir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/invoke", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "contract_does_not_exist";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user4",
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(script_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::TransactionExecutionError(TransactionExecutionErrorData { transaction_index: 0, execution_error: "Transaction execution has failed:
        [..]
        [..]: Error in the called contract ([..]):
        Requested contract address [..] is not deployed.
        " })))
        command: script run
        status: success
        "#},
    );
}

#[test]
fn test_wrong_function_name() {
    let script_dir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/invoke", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "wrong_function_name";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user4",
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(script_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::TransactionExecutionError(TransactionExecutionErrorData { transaction_index: 0, execution_error: "Transaction execution has failed:
        [..]
        [..]: Error in the called contract ([..]):
        Entry point EntryPointSelector([..]) not found in contract.
        " })))
        command: script run
        status: success
        "#},
    );
}

#[test]
fn test_wrong_calldata() {
    let script_dir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/invoke", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "wrong_calldata";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user4",
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(script_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::TransactionExecutionError(TransactionExecutionErrorData { transaction_index: 0, execution_error: "Transaction execution has failed:
        [..]
        [..]: Error in the called contract ([..]):
        Execution failed. Failure reason: [..] ('Failed to deserialize param #2').
        " })))
        command: script run
        status: success
        "#},
    );
}
