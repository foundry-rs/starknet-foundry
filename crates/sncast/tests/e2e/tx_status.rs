use crate::helpers::constants::URL;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;

const SUCCEEDED_TX_HASH: &str =
    "0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1";
const REVERTED_TX_HASH: &str = "0x00ae35dacba17cde62b8ceb12e3b18f4ab6e103fa2d5e3d9821cb9dc59d59a3c";

#[tokio::test]
async fn test_incorrect_transaction_hash() {
    let args = vec!["tx-status", "0x1", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: tx-status
        Error: Failed to get transaction status: Transaction with provided hash was not found (does not exist)
        "},
    );
}

#[tokio::test]
async fn test_succeeded() {
    let args = vec!["tx-status", SUCCEEDED_TX_HASH, "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Transaction status retrieved
        
        Finality Status:  Accepted on L1
        Execution Status: Succeeded
    "});
}

#[tokio::test]
async fn test_reverted() {
    let args = vec!["tx-status", REVERTED_TX_HASH, "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Transaction status retrieved
        
        Finality Status:  Accepted on L1
        Execution Status: Reverted
    "});
}
