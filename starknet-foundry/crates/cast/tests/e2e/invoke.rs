use crate::helpers::constants::{ACCOUNT, MAP_CONTRACT_ADDRESS_V1, MAP_CONTRACT_ADDRESS_V2};
use crate::helpers::fixtures::{default_cli_args, get_transaction_hash, get_transaction_receipt};
use crate::helpers::runner::runner;
use indoc::indoc;
use starknet::core::types::TransactionReceipt::Invoke;
use test_case::test_case;

#[test_case(MAP_CONTRACT_ADDRESS_V1, "user1" ; "when cairo1 contract")]
#[test_case(MAP_CONTRACT_ADDRESS_V2, "user2" ; "when cairo2 contract")]
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
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        ACCOUNT,
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

#[test_case(MAP_CONTRACT_ADDRESS_V1, "user1" ; "when cairo1 contract")]
#[test_case(MAP_CONTRACT_ADDRESS_V2, "user2" ; "when cairo2 contract")]
fn test_wrong_function_name(contract_address: &str, account: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        account,
        "invoke",
        "--contract-address",
        contract_address,
        "--entry-point-name",
        "nonexistent_put",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        error: An error occurred in the called contract
    "#});
}

#[test_case(MAP_CONTRACT_ADDRESS_V1, "user1" ; "when cairo1 contract")]
#[test_case(MAP_CONTRACT_ADDRESS_V2, "user2" ; "when cairo2 contract")]
fn test_wrong_calldata(contract_address: &str, account: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        account,
        "invoke",
        "--contract-address",
        contract_address,
        "--entry-point-name",
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
    assert!(stderr_str.contains(contract_address));
}

#[test_case(MAP_CONTRACT_ADDRESS_V1, "user1" ; "when cairo1 contract")]
#[test_case(MAP_CONTRACT_ADDRESS_V2, "user2" ; "when cairo2 contract")]
fn test_too_low_max_fee(contract_address: &str, account: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--account",
        account,
        "invoke",
        "--contract-address",
        contract_address,
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
