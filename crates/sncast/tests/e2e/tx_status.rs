use crate::helpers::constants::URL;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;

const SUCCEEDED_TX_HASH: &str =
    "0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1";
const REVERTED_TX_HASH: &str = "0x00ae35dacba17cde62b8ceb12e3b18f4ab6e103fa2d5e3d9821cb9dc59d59a3c";

#[tokio::test]
async fn test_incorrect_transaction_hash() {
    let args = vec!["--url", URL, "tx-status", "0x1"];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: tx-status
        error: Failed to get transaction status: TransactionHashNotFound
        "},
    );
}

#[tokio::test]
async fn test_succeeded() {
    let args = vec!["--url", URL, "tx-status", SUCCEEDED_TX_HASH];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        command: tx-status
        execution_status: Succeeded
        finality_status: AcceptedOnL1
    "});
}

#[tokio::test]
async fn test_reverted() {
    let args = vec!["--url", URL, "tx-status", REVERTED_TX_HASH];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        command: tx-status
        execution_status: Reverted
        finality_status: AcceptedOnL1
    "});
}
