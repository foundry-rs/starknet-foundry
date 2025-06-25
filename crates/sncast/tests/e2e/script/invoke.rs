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
        Success: Script execution completed
        
        Status: success
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
        ScriptCommandError::WaitForTransactionError(WaitForTransactionError::TransactionError(TransactionError::Reverted(ErrorData { msg: "Transaction execution has failed:
        [..]
        [..]: Error in the called contract ([..]):
        Requested contract address [..] is not deployed.
        " })))
        Success: Script execution completed
        
        Status: success
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

    // TODO(#3116): Change message to string after issue with undecoded felt is resolved.
    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::TransactionExecutionError(TransactionExecutionErrorData { transaction_index: 0, execution_error: ContractExecutionError::Nested(&ContractExecutionErrorInner { contract_address: [..], class_hash: [..], selector: [..], error: ContractExecutionError::Nested(&ContractExecutionErrorInner { contract_address: [..], class_hash: [..], selector: [..], error: ContractExecutionError::Nested(&ContractExecutionErrorInner { contract_address: [..], class_hash: [..], selector: [..], error: ContractExecutionError::Message("Transaction execution has failed:
        0: Error in the called contract (contract address: 0x03ffc270312cbefaf2fb4a88e97cc186797bada41a291331186ec5ca316e32fa, class hash: 0x05b4b537eaa2399e3aa99c4e2e0208ebd6c71bc1467938cd52c798c601e43564, selector: [..]):
        Execution failed. Failure reason:
        Error in contract (contract address: [..], class hash: [..], selector: [..]):
        Error in contract (contract address: [..], class hash: [..], selector: [..]):
        [..] ('ENTRYPOINT_NOT_FOUND').
         ["0x454e545259504f494e545f4e4f545f464f554e44"]") }) }) }) })))
        Success: Script execution completed
        
        Status: success
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

    // TODO(#3116): Change message to string after issue with undecoded felt is resolved.
    assert_stdout_contains(
        output,
        indoc! {r#"
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::TransactionExecutionError(TransactionExecutionErrorData { transaction_index: 0, execution_error: ContractExecutionError::Nested(&ContractExecutionErrorInner { contract_address: [..], class_hash: [..], selector: [..], error: ContractExecutionError::Nested(&ContractExecutionErrorInner { contract_address: [..], class_hash: [..], selector: [..], error: ContractExecutionError::Nested(&ContractExecutionErrorInner { contract_address: [..], class_hash: [..], selector: [..], error: ContractExecutionError::Message("Transaction execution has failed:
        0: Error in the called contract (contract address: [..], class hash: [..], selector: [..]):
        Execution failed. Failure reason:
        Error in contract (contract address: [..], class hash: [..], selector: [..]):
        Error in contract (contract address: [..], class hash: [..], selector: [..]):
        [..] ('Failed to deserialize param #2').
         ["0x4661696c656420746f20646573657269616c697a6520706172616d202332"]") }) }) }) })))
        Success: Script execution completed
        
        Status: success
        "#},
    );
}
