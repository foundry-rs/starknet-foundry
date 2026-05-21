use crate::helpers::constants::URL;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};

const INVOKE_TX_HASH: &str = "0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1";

#[tokio::test]
async fn test_get_tx_receipt() {
    let args = vec!["get", "tx-receipt", INVOKE_TX_HASH, "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        Success: Transaction receipt retrieved

        Type:                 INVOKE
        Transaction Hash:     0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1
        Finality Status:      Accepted on L1
        Execution Status:     Succeeded
        "},
    );
}

#[tokio::test]
async fn test_get_tx_receipt_alias() {
    let args = vec!["get", "transaction-receipt", INVOKE_TX_HASH, "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stdout_contains(output, "Success: Transaction receipt retrieved");
}

#[tokio::test]
async fn test_json_output() {
    let args = vec!["--json", "get", "tx-receipt", INVOKE_TX_HASH, "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();
    let stdout = output.get_output().stdout.clone();

    let json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(json["command"], "get tx-receipt");
    assert_eq!(json["type"], "response");
    assert!(
        json["transaction_hash"]
            .as_str()
            .unwrap()
            .starts_with("0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1")
    );
    assert_eq!(json["finality_status"], "ACCEPTED_ON_L1");
    assert!(json["actual_fee"].is_object());
    assert!(json["execution_resources"].is_object());
    assert!(json["events"].is_array());
    assert!(json["messages_sent"].is_array());
}

#[tokio::test]
async fn test_nonexistent_transaction() {
    let args = vec!["get", "tx-receipt", "0x1", "--url", URL];
    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: get tx-receipt
        Error: Transaction with provided hash was not found (does not exist)
        "},
    );
}
