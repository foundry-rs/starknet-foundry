use crate::helpers::constants::{REVERTED_TX_HASH, SUCCEEDED_TX_HASH, URL};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};

#[tokio::test]
async fn test_get_tx_receipt() {
    let args = vec!["get", "tx-receipt", SUCCEEDED_TX_HASH, "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Transaction receipt retrieved

        Type:                 INVOKE
        Transaction Hash:     0x04cba686fa76bfa4b4ac788bf2ca9bfac3dd354561f2621c2ac7cf17fa46f75a
        Finality Status:      Accepted on L1
        Execution Status:     Succeeded
        Actual Fee:           0x19470f45bbcbe0 FRI
        L1 Gas Consumed:      0
        L1 Data Gas Consumed: 288
        L2 Gas Consumed:      889345
        Messages Sent:        0
        Events:               2
        Block Hash:           0x018d7f834b90964651926b59347dfd33b121fd5709a45b6de7e47a9997bb7700
        Block Number:         7776133
    "});
}

#[tokio::test]
async fn test_get_tx_receipt_alias() {
    let args = vec![
        "get",
        "transaction-receipt",
        SUCCEEDED_TX_HASH,
        "--url",
        URL,
    ];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stdout_contains(output, "Success: Transaction receipt retrieved");
}

#[tokio::test]
async fn test_get_tx_receipt_reverted() {
    let args = vec!["get", "tx-receipt", REVERTED_TX_HASH, "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Transaction receipt retrieved

        Type:                 INVOKE
        Transaction Hash:     0x00ae35dacba17cde62b8ceb12e3b18f4ab6e103fa2d5e3d9821cb9dc59d59a3c
        Finality Status:      Accepted on L1
        Execution Status:     Reverted
        Revert Reason:        Error in the called contract (0x0143fe26927dd6a302522ea1cd6a821ab06b3753194acee38d88a85c93b3cbc6):
        Error at pc=0:4271:
        Got an exception while executing a hint: Execution failed. Failure reason: 0x7374616c65207265706f7274 ('stale report').
        Cairo traceback (most recent call last):
        Unknown location (pc=0:65)
        Unknown location (pc=0:1787)
        Unknown location (pc=0:2271)
        Unknown location (pc=0:2984)
        Unknown location (pc=0:3611)

        Error in the called contract (0x00132303a40ae2f271f4e1b707596a63f6f2921c4d400b38822548ed1bb0cbe0):
        Execution failed. Failure reason: 0x7374616c65207265706f7274 ('stale report').

        Actual Fee:           0xb2a72b1ed0f0906 FRI
        L1 Gas Consumed:      0
        L1 Data Gas Consumed: 0
        L2 Gas Consumed:      0
        Messages Sent:        0
        Events:               1
        Block Hash:           0x002d89f8c1dc6b4f3cdfdcf66dd0d337098f447879618066c3f8e18438429f58
        Block Number:         69604
    "});
}

#[tokio::test]
async fn test_json_output() {
    let args = vec![
        "--json",
        "get",
        "tx-receipt",
        SUCCEEDED_TX_HASH,
        "--url",
        URL,
    ];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();
    let stdout = output.get_output().stdout.clone();

    let json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();

    assert_eq!(json["command"], "get tx-receipt");
    assert_eq!(json["type"], "response");
    assert_eq!(
        json["transaction_hash"],
        "0x4cba686fa76bfa4b4ac788bf2ca9bfac3dd354561f2621c2ac7cf17fa46f75a"
    );
    assert_eq!(json["finality_status"], "ACCEPTED_ON_L1");
    assert_eq!(json["execution_status"], "SUCCEEDED");
    assert_eq!(
        json["block_hash"],
        "0x18d7f834b90964651926b59347dfd33b121fd5709a45b6de7e47a9997bb7700"
    );
    assert_eq!(json["block_number"], 7_776_133);

    assert_eq!(json["actual_fee"]["amount"], "0x19470f45bbcbe0");
    assert_eq!(json["actual_fee"]["unit"], "FRI");

    assert_eq!(json["execution_resources"]["l1_gas"], 0);
    assert_eq!(json["execution_resources"]["l1_data_gas"], 288);
    assert_eq!(json["execution_resources"]["l2_gas"], 889_345);

    assert_eq!(json["messages_sent"].as_array().unwrap().len(), 0);
    assert_eq!(json["events"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_json_output_reverted() {
    let args = vec![
        "--json",
        "get",
        "tx-receipt",
        REVERTED_TX_HASH,
        "--url",
        URL,
    ];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();
    let stdout = output.get_output().stdout.clone();

    let json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();

    assert_eq!(json["execution_status"], "REVERTED");
    assert!(
        json["revert_reason"]
            .as_str()
            .unwrap()
            .contains("stale report")
    );
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
