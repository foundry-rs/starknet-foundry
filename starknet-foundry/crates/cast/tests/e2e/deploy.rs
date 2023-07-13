use crate::helpers::constants::MAP_CLASS_HASH;
use crate::helpers::fixtures::{default_cli_args, get_transaction_hash, get_transaction_receipt};
use crate::helpers::runner::runner;
use indoc::indoc;
use starknet::core::types::TransactionReceipt::Invoke;

#[tokio::test]
async fn test_happy_case() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--int-format",
        "--json",
        "deploy",
        "--class-hash",
        MAP_CLASS_HASH,
        "--salt",
        "0x2",
        "--unique",
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
async fn test_contract_not_declared() {
    let mut args = default_cli_args();
    args.append(&mut vec!["deploy", "--class-hash", "0x1"]);

    let snapbox = runner(&args);
    let output = String::from_utf8(snapbox.assert().success().get_output().stderr.clone()).unwrap();

    assert!(output.contains("Class with hash 0x1 is not declared."));
}

#[tokio::test]
async fn test_contract_already_deployed() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "deploy",
        "--class-hash",
        MAP_CLASS_HASH,
        "--salt",
        "0x1",
    ]);

    let snapbox = runner(&args);
    let output = String::from_utf8(snapbox.assert().success().get_output().stderr.clone()).unwrap();

    assert!(output.contains("StarknetErrorCode.CONTRACT_ADDRESS_UNAVAILABLE"));
}

#[tokio::test]
async fn test_too_low_max_fee() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "deploy",
        "--class-hash",
        MAP_CLASS_HASH,
        "--salt",
        "0x2",
        "--unique",
        "--max-fee",
        "1",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        error: Transaction has been rejected
    "#});
}
