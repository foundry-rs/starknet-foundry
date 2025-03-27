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
        ScriptCommandError::WaitForTransactionError(WaitForTransactionError::TransactionError(TransactionError::Reverted(ErrorData { msg: "Transaction execution has failed:
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

    // TODO(#3116): Change message to string after issue with undecoded felt is resolved.
    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]
        command: script run
        status: success
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::TransactionExecutionError(TransactionExecutionErrorData { transaction_index: 0, execution_error: ContractExecutionError::Nested(&ContractExecutionErrorInner { contract_address: 1221099072199994236942962469331298328195197638723172636491022210224930948700, class_hash: 1808826510752300994761702802901481645174578400828494258410687349947481600762, selector: 617075754465154585683856897856256838130216341506379215893724690153393808813, error: ContractExecutionError::Nested(&ContractExecutionErrorInner { contract_address: 1221099072199994236942962469331298328195197638723172636491022210224930948700, class_hash: 1808826510752300994761702802901481645174578400828494258410687349947481600762, selector: 617075754465154585683856897856256838130216341506379215893724690153393808813, error: ContractExecutionError::Nested(&ContractExecutionErrorInner { contract_address: 1188339057176664209280699478863418064314752708310222779632491631365185262369, class_hash: 363194768697721961899319994258180961155471351945520595315225700771410817032, selector: 858433424616582048654902318516638408354213827276012205855287183724364473156, error: ContractExecutionError::Message("Transaction execution has failed:
        0: Error in the called contract (contract address: 0x03ffc270312cbefaf2fb4a88e97cc186797bada41a291331186ec5ca316e32fa, class hash: 0x02b31e19e45c06f29234e06e2ee98a9966479ba3067f8785ed972794fdb0065c, selector: 0x015d40a3d6ca2ac30f4031e42be28da9b056fef9bb7357ac5e85627ee876e5ad):
        Execution failed. Failure reason:
        Error in contract (contract address: 0x03ffc270312cbefaf2fb4a88e97cc186797bada41a291331186ec5ca316e32fa, class hash: 0x02b31e19e45c06f29234e06e2ee98a9966479ba3067f8785ed972794fdb0065c, selector: 0x015d40a3d6ca2ac30f4031e42be28da9b056fef9bb7357ac5e85627ee876e5ad):
        Error in contract (contract address: 0x00cd8f9ab31324bb93251837e4efb4223ee195454f6304fcfcb277e277653008, class hash: 0x02a09379665a749e609b4a8459c86fe954566a6beeaddd0950e43f6c700ed321, selector: 0x01e5db2962abdd13feabdd2ba5a988ba5a790f34d4d6f072c3aa59872520a344):
        0x454e545259504f494e545f4e4f545f464f554e44 ('ENTRYPOINT_NOT_FOUND').
         ["0x454e545259504f494e545f4e4f545f464f554e44"]") }) }) }) })))
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
        command: script run
        status: success
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::TransactionExecutionError(TransactionExecutionErrorData { transaction_index: 0, execution_error: ContractExecutionError::Nested(&ContractExecutionErrorInner { contract_address: [..], class_hash: [..], selector: [..], error: ContractExecutionError::Nested(&ContractExecutionErrorInner { contract_address: [..], class_hash: [..], selector: [..], error: ContractExecutionError::Nested(&ContractExecutionErrorInner { contract_address: [..], class_hash: [..], selector: [..], error: ContractExecutionError::Message("Transaction execution has failed:
        0: Error in the called contract (contract address: [..], class hash: [..], selector: [..]):
        Execution failed. Failure reason:
        Error in contract (contract address: [..], class hash: [..], selector: [..]):
        Error in contract (contract address: [..], class hash: [..], selector: [..]):
        [..] ('Failed to deserialize param #2').
         ["0x4661696c656420746f20646573657269616c697a6520706172616d202332"]") }) }) }) })))
        "#},
    );
}
