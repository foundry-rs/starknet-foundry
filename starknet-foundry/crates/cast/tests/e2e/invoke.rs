use crate::helpers::constants::MAP_CONTRACT_ADDRESS;
use crate::helpers::fixtures::{default_cli_args, get_transaction_hash, get_transaction_receipt};
use crate::helpers::runner::runner;
use indoc::indoc;
use starknet::core::types::TransactionReceipt::Invoke;

static USERNAME: &str = "user2";

#[tokio::test]
async fn test_happy_case() {
    let args = default_cli_args(USERNAME);
    let mut args: Vec<&str> = args.iter().map(String::as_str).collect();
    args.append(&mut vec![
        "--int-format",
        "--json",
        "invoke",
        "--contract-address",
        MAP_CONTRACT_ADDRESS,
        "--entry-point-name",
        "put",
        "--calldata",
        "0x1 0x2",
        "--max-fee",
        "999999999999",
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Invoke(_)));
}

#[tokio::test]
async fn test_contract_does_not_exist() {
    let args = default_cli_args(USERNAME);
    let mut args: Vec<&str> = args.iter().map(String::as_str).collect();
    args.append(&mut vec![
        "invoke",
        "--contract-address",
        "0x1",
        "--entry-point-name",
        "put",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        error: There is no contract at the specified address
    "#});
}

#[tokio::test]
async fn test_wrong_function_name() {
    let args = default_cli_args(USERNAME);
    let mut args: Vec<&str> = args.iter().map(String::as_str).collect();
    args.append(&mut vec![
        "invoke",
        "--contract-address",
        MAP_CONTRACT_ADDRESS,
        "--entry-point-name",
        "nonexistent_put",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        error: An error occurred in the called contract
    "#});
}

#[tokio::test]
async fn test_wrong_calldata() {
    let args = default_cli_args(USERNAME);
    let mut args: Vec<&str> = args.iter().map(String::as_str).collect();
    args.append(&mut vec![
        "invoke",
        "--contract-address",
        MAP_CONTRACT_ADDRESS,
        "--entry-point-name",
        "put",
        "--calldata",
        "0x1",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        error: Error at pc=0:81:
        Got an exception while executing a hint.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)

        Error in the called contract (0x38b7b9507ccf73d79cb42c2cc4e58cf3af1248f342112879bfdf5aa4f606cc9):
        Execution was reverted; failure reason: [0x496e70757420746f6f2073686f727420666f7220617267756d656e7473].
    "#});
}

#[tokio::test]
async fn test_too_low_max_fee() {
    let args = default_cli_args(USERNAME);
    let mut args: Vec<&str> = args.iter().map(String::as_str).collect();
    args.append(&mut vec![
        "invoke",
        "--contract-address",
        MAP_CONTRACT_ADDRESS,
        "--entry-point-name",
        "put",
        "--calldata",
        "0x1 0x2",
        "--max-fee",
        "1",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        error: Transaction has been rejected
    "#});
}
