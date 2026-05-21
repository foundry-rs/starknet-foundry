use crate::helpers::constants::{REVERTED_TX_HASH, SUCCEEDED_TX_HASH, URL};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};

#[tokio::test]
async fn test_incorrect_transaction_hash() {
    let args = vec!["get", "tx-status", "0x1", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: get tx-status
        Error: Transaction with provided hash was not found (does not exist)
        "},
    );
}

#[tokio::test]
async fn test_succeeded_old_command() {
    let args = vec!["tx-status", SUCCEEDED_TX_HASH, "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        [WARNING] `sncast tx-status` has moved to `sncast get tx-status`. `sncast tx-status` will be removed in the next version.

        Success: Transaction status retrieved
        
        Finality Status:  Accepted on L1
        Execution Status: Succeeded
    "});
}

#[tokio::test]
async fn test_succeeded() {
    let args = vec!["get", "tx-status", SUCCEEDED_TX_HASH, "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Transaction status retrieved

        Finality Status:  Accepted on L1
        Execution Status: Succeeded
    "});
}

#[tokio::test]
async fn test_succeeded_alias() {
    let args = vec!["get", "transaction-status", SUCCEEDED_TX_HASH, "--url", URL];
    let snapbox = runner(&args);

    let output = snapbox.assert().success();
    assert_stdout_contains(output, "Success: Transaction status retrieved");
}

#[tokio::test]
async fn test_reverted() {
    let args = vec!["get", "tx-status", REVERTED_TX_HASH, "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Transaction status retrieved
        
        Finality Status:  Accepted on L1
        Execution Status: Reverted
    "});
}
