use crate::helpers::constants::{SCRIPTS_DIR, URL, devnet_url};
use crate::helpers::fixtures::copy_script_directory_to_tempdir;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[tokio::test]
async fn test_tx_status_status_reverted() {
    let tempdir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/tx_status",
        Vec::<String>::new(),
    );

    let script_name = "status_reverted";
    let url = devnet_url();
    let args = vec!["script", "run", &script_name, "--url", &url];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        finality_status: AcceptedOnL1, execution_status: Reverted
        TxStatusResult { finality_status: FinalityStatus::AcceptedOnL1(()), execution_status: Option::Some(ExecutionStatus::Reverted(())) }
        Success: Script execution completed
        
        Status: success
        "},
    );
}

#[tokio::test]
async fn test_tx_status_status_succeeded() {
    let tempdir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/tx_status",
        Vec::<String>::new(),
    );

    let script_name = "status_succeeded";
    let url = devnet_url();
    let args = vec!["script", "run", &script_name, "--url", &url];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        finality_status: AcceptedOnL1, execution_status: Succeeded
        TxStatusResult { finality_status: FinalityStatus::AcceptedOnL1(()), execution_status: Option::Some(ExecutionStatus::Succeeded(())) }
        Success: Script execution completed
        
        Status: success
        "},
    );
}

#[tokio::test]
async fn test_tx_status_incorrect_transaction_hash() {
    let tempdir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/tx_status",
        Vec::<String>::new(),
    );

    let script_name = "incorrect_transaction_hash";
    let url = devnet_url();
    let args = vec!["script", "run", &script_name, "--url", &url];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::TransactionHashNotFound(())))
        Success: Script execution completed
        
        Status: success
        "},
    );
}
