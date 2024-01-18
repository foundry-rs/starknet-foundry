use crate::helpers::constants::ACCOUNT;
use crate::helpers::fixtures::{
    default_cli_args, from_env, get_transaction_hash, get_transaction_receipt,
};
use crate::helpers::runner::runner;
use indoc::indoc;
use starknet::core::types::TransactionReceipt::Invoke;

#[tokio::test]
async fn test_happy_case() {
    let contract_address = from_env("CAST_MAP_ADDRESS").unwrap();
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user2",
        "--int-format",
        "--json",
        "invoke",
        "--contract-address",
        &contract_address,
        "--function",
        "put",
        "--calldata",
        "0x1 0x2",
        "--max-fee",
        "99999999999999999",
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Invoke(_)));
}

#[tokio::test]
async fn test_contract_does_not_exist() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        ACCOUNT,
        "invoke",
        "--contract-address",
        "0x1",
        "--function",
        "put",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r"
        command: invoke
        error: An error occurred in the called contract [..]
    "});
}

#[test]
fn test_wrong_function_name() {
    let contract_address = from_env("CAST_MAP_ADDRESS").unwrap();
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user2",
        "invoke",
        "--contract-address",
        &contract_address,
        "--function",
        "nonexistent_put",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r"
        command: invoke
        error: An error occurred in the called contract [..]
    "});
}

#[test]
fn test_wrong_calldata() {
    let contract_address = from_env("CAST_MAP_ADDRESS").unwrap();
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user5",
        "invoke",
        "--contract-address",
        &contract_address,
        "--function",
        "put",
        "--calldata",
        "0x1",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r"
        command: invoke
        error: An error occurred in the called contract [..]
    "});
}

#[test]
fn test_too_low_max_fee() {
    let contract_address = from_env("CAST_MAP_ADDRESS").unwrap();
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        "user2",
        "--wait",
        "invoke",
        "--contract-address",
        &contract_address,
        "--function",
        "put",
        "--calldata",
        "0x1",
        "0x2",
        "--max-fee",
        "1",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r"
        command: invoke
        error: Max fee is smaller than the minimal transaction cost
    "});
}
