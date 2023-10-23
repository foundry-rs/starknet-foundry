use crate::helpers::constants::ACCOUNT;
use crate::helpers::fixtures::{
    default_cli_args, from_env, get_transaction_hash, get_transaction_receipt,
};
use crate::helpers::runner::runner;
use indoc::indoc;
use starknet::core::types::TransactionReceipt::Invoke;

#[tokio::test]
async fn test_happy_case() {
    let class_hash = from_env("CAST_MAP_CLASS_HASH").unwrap();
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user2",
        "--int-format",
        "--json",
        "deploy",
        "--class-hash",
        &class_hash,
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
async fn test_happy_case_with_constructor() {
    let class_hash = from_env("CAST_WITH_CONSTRUCTOR_CLASS_HASH").unwrap();
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user4",
        "--int-format",
        "--json",
        "deploy",
        "--class-hash",
        &class_hash,
        "--constructor-calldata",
        "0x1 0x1 0x0",
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Invoke(_)));
}

#[test]
fn test_wrong_calldata() {
    let class_hash = from_env("CAST_WITH_CONSTRUCTOR_CLASS_HASH").unwrap();
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user4",
        "deploy",
        "--class-hash",
        &class_hash,
        "--constructor-calldata",
        "0x1 0x1",
    ]);

    let snapbox = runner(&args);
    let output = String::from_utf8(snapbox.assert().success().get_output().stderr.clone()).unwrap();

    assert!(output.contains("error: "));
    assert!(output.contains("Error in the called contract"));
}

#[tokio::test]
async fn test_contract_not_declared() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        ACCOUNT,
        "deploy",
        "--class-hash",
        "0x1",
    ]);

    let snapbox = runner(&args);
    let output = String::from_utf8(snapbox.assert().get_output().stderr.clone()).unwrap();

    assert!(output.contains("Class with hash 0x1 is not declared."));
}

#[test]
fn test_contract_already_deployed() {
    let class_hash = from_env("CAST_MAP_CLASS_HASH").unwrap();
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user1",
        "deploy",
        "--class-hash",
        &class_hash,
        "--salt",
        "0x1",
        "--unique",
    ]);

    let snapbox = runner(&args);
    let output = String::from_utf8(snapbox.assert().get_output().stderr.clone()).unwrap();

    assert!(output.contains("StarknetErrorCode.CONTRACT_ADDRESS_UNAVAILABLE"));
}

#[test]
fn test_too_low_max_fee() {
    let class_hash = from_env("CAST_MAP_CLASS_HASH").unwrap();
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user2",
        "--wait",
        "deploy",
        "--class-hash",
        &class_hash,
        "--salt",
        "0x2",
        "--unique",
        "--max-fee",
        "1",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
        command: deploy
        error: Transaction has been rejected
    "#});
}
