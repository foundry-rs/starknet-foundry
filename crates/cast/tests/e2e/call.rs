use crate::helpers::constants::{MAP_CONTRACT_ADDRESS_V1, MAP_CONTRACT_ADDRESS_V2};
use crate::helpers::fixtures::{default_cli_args, invoke_map_contract};
use crate::helpers::runner::runner;
use indoc::indoc;
use test_case::test_case;

#[test_case(MAP_CONTRACT_ADDRESS_V1 ; "when cairo1 contract")]
#[test_case(MAP_CONTRACT_ADDRESS_V2 ; "when cairo2 contract")]
fn test_happy_case(contract_address: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--json",
        "call",
        "--contract-address",
        contract_address,
        "--function",
        "get",
        "--calldata",
        "0x0",
        "--block-id",
        "latest",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
{
  "command": "call",
  "response": "[0x0]"
}
"#});
}

#[test_case(MAP_CONTRACT_ADDRESS_V1, "user1" ; "when cairo1 contract")]
#[test_case(MAP_CONTRACT_ADDRESS_V2, "user2" ; "when cairo2 contract")]
#[tokio::test]
async fn test_call_after_storage_changed(contract_address: &str, account: &str) {
    invoke_map_contract("0x2", "0x3", account, contract_address).await;

    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        contract_address,
        "--function",
        "get",
        "--calldata",
        "0x2",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
        command: call
        response: [0x3]
    "#});
}

#[tokio::test]
async fn test_contract_does_not_exist() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        "0x1",
        "--function",
        "get",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
        command: call
        error: There is no contract at the specified address
    "#});
}

#[test_case(MAP_CONTRACT_ADDRESS_V1 ; "when cairo1 contract")]
#[test_case(MAP_CONTRACT_ADDRESS_V2 ; "when cairo2 contract")]
fn test_wrong_function_name(contract_address: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        contract_address,
        "--function",
        "nonexistent_get",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
        command: call
        error: An error occurred in the called contract
    "#});
}

#[test_case(MAP_CONTRACT_ADDRESS_V1 ; "when cairo1 contract")]
#[test_case(MAP_CONTRACT_ADDRESS_V2 ; "when cairo2 contract")]
fn test_wrong_calldata(contract_address: &str) {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        contract_address,
        "--function",
        "get",
        "--calldata",
        "0x1 0x2",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
        command: call
        error: Execution was reverted; failure reason: [0x496e70757420746f6f206c6f6e6720666f7220617267756d656e7473].
    "#});
}
