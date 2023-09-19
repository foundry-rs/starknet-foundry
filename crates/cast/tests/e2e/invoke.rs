use crate::helpers::constants::ACCOUNT;
use crate::helpers::fixtures::{
    convert_to_hex, default_cli_args, from_env, get_transaction_hash, get_transaction_receipt,
};
use crate::helpers::runner::runner;
use indoc::indoc;
use starknet::core::types::TransactionReceipt::Invoke;
use test_case::test_case;

#[test_case(from_env("CAST_MAP_V1_ADDRESS").unwrap().as_str(), "user1" ; "when cairo1 contract")]
#[test_case(from_env("CAST_MAP_V2_ADDRESS").unwrap().as_str(), "user2" ; "when cairo2 contract")]
#[tokio::test]
async fn test_happy_case(contract_address: &str, account: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        account,
        "--int-format",
        "--json",
        "invoke",
        "--contract-address",
        contract_address,
        "--function",
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

    snapbox.assert().stderr_matches(indoc! {r#"
        command: invoke
        error: Contract not found
    "#});
}

#[test_case(from_env("CAST_MAP_V1_ADDRESS").unwrap().as_str(), "user1" ; "when cairo1 contract")]
#[test_case(from_env("CAST_MAP_V2_ADDRESS").unwrap().as_str(), "user2" ; "when cairo2 contract")]
fn test_wrong_function_name(contract_address: &str, account: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        account,
        "invoke",
        "--contract-address",
        contract_address,
        "--function",
        "nonexistent_put",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
        command: invoke
        error: Contract error
    "#});
}

#[test_case(from_env("CAST_MAP_V1_ADDRESS").unwrap().as_str(), "user1" ; "when cairo1 contract")]
#[test_case(from_env("CAST_MAP_V2_ADDRESS").unwrap().as_str(), "user2" ; "when cairo2 contract")]
fn test_wrong_calldata(contract_address: &str, account: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        account,
        "invoke",
        "--contract-address",
        contract_address,
        "--function",
        "put",
        "--calldata",
        "0x1",
    ]);

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stderr_str =
        std::str::from_utf8(&out.stderr).expect("failed to convert command output to string");

    assert!(stderr_str.contains("Error in the called contract"));
    assert!(stderr_str.contains(&convert_to_hex(contract_address)));
}

#[test_case(from_env("CAST_MAP_V1_ADDRESS").unwrap().as_str(), "user1" ; "when cairo1 contract")]
#[test_case(from_env("CAST_MAP_V2_ADDRESS").unwrap().as_str(), "user2" ; "when cairo2 contract")]
fn test_too_low_max_fee(contract_address: &str, account: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        account,
        "--wait",
        "invoke",
        "--contract-address",
        contract_address,
        "--function",
        "put",
        "--calldata",
        "0x1 0x2",
        "--max-fee",
        "1",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
        command: invoke
        error: Transaction has been rejected
    "#});
}
