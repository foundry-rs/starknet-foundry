use crate::helpers::constants::ACCOUNT;
use crate::helpers::fixtures::{default_cli_args, get_transaction_hash, get_transaction_receipt};
use crate::helpers::runner::runner;
use indoc::indoc;
use starknet::core::types::TransactionReceipt::Invoke;
use std::env;
use test_case::test_case;

#[test_case(&env::var("MAP_V1_CLASS_HASH").expect("MAP_V1_CLASS_HASH not available in env!"), "user1" ; "when cairo1 contract")]
#[test_case(&env::var("MAP_V2_CLASS_HASH").expect("MAP_V2_CLASS_HASH not available in env!"), "user2" ; "when cairo2 contract")]
#[tokio::test]
async fn test_happy_case(class_hash: &str, account: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        account,
        "--int-format",
        "--json",
        "deploy",
        "--class-hash",
        class_hash,
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

#[test_case(&env::var("WITH_CONSTRUCTOR_V1_CLASS_HASH").expect("CONTRACT_WITH_CONSTRUCTOR_V1_CLASS_HASH not available in env!"), "user3" ; "when cairo1 contract")]
#[test_case(&env::var("WITH_CONSTRUCTOR_V2_CLASS_HASH").expect("CONTRACT_WITH_CONSTRUCTOR_V2_CLASS_HASH not available in env!"), "user4" ; "when cairo2 contract")]
#[tokio::test]
async fn test_happy_case_with_constructor(class_hash: &str, account: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        account,
        "--int-format",
        "--json",
        "deploy",
        "--class-hash",
        class_hash,
        "--constructor-calldata",
        "0x1 0x1 0x0",
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Invoke(_)));
}

#[test_case(&env::var("WITH_CONSTRUCTOR_V1_CLASS_HASH").expect("CONTRACT_WITH_CONSTRUCTOR_V1_CLASS_HASH not available in env!"), "user3" ; "when cairo1 contract")]
#[test_case(&env::var("WITH_CONSTRUCTOR_V2_CLASS_HASH").expect("CONTRACT_WITH_CONSTRUCTOR_V2_CLASS_HASH not available in env!"), "user4" ; "when cairo2 contract")]
fn test_wrong_calldata(class_hash: &str, account: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        account,
        "deploy",
        "--class-hash",
        class_hash,
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

#[test_case(&env::var("MAP_V1_CLASS_HASH").expect("MAP_V1_CLASS_HASH not available in env!"), "user1" ; "when cairo1 contract")]
#[test_case(&env::var("MAP_V2_CLASS_HASH").expect("MAP_V2_CLASS_HASH not available in env!"), "user2" ; "when cairo2 contract")]
fn test_contract_already_deployed(class_hash: &str, account: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        account,
        "deploy",
        "--class-hash",
        class_hash,
        "--salt",
        "0x1",
        "--unique",
    ]);

    let snapbox = runner(&args);
    let output = String::from_utf8(snapbox.assert().get_output().stderr.clone()).unwrap();

    assert!(output.contains("StarknetErrorCode.CONTRACT_ADDRESS_UNAVAILABLE"));
}

#[test_case(&env::var("MAP_V1_CLASS_HASH").expect("MAP_V1_CLASS_HASH not available in env!"), "user1" ; "when cairo1 contract")]
#[test_case(&env::var("MAP_V2_CLASS_HASH").expect("MAP_V2_CLASS_HASH not available in env!"), "user2" ; "when cairo2 contract")]
fn test_too_low_max_fee(class_hash: &str, account: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        account,
        "--wait",
        "deploy",
        "--class-hash",
        class_hash,
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
