use crate::helpers::constants::MAP_CONTRACT_ADDRESS;
use crate::helpers::fixtures::{default_cli_args, invoke_map_contract};
use crate::helpers::runner::runner;
use indoc::indoc;

#[tokio::test]
async fn test_happy_case() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "--json",
        "call",
        "--contract-address",
        MAP_CONTRACT_ADDRESS,
        "--function-name",
        "get",
        "--calldata",
        "0x0",
        "--block-id",
        "latest",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
{
  "command": "Call",
  "response": "[FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000000 }]"
}
"#});
}

#[tokio::test]
async fn test_call_after_storage_changed() {
    invoke_map_contract("0x2", "0x3").await;

    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        MAP_CONTRACT_ADDRESS,
        "--function-name",
        "get",
        "--calldata",
        "0x2",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
        command: Call
        response: [FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000003 }]
    "#});
}

#[tokio::test]
async fn test_contract_does_not_exist() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        "0x1",
        "--function-name",
        "get",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        error: There is no contract at the specified address
    "#});
}

#[tokio::test]
async fn test_wrong_function_name() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        MAP_CONTRACT_ADDRESS,
        "--function-name",
        "nonexistent_get",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        error: An error occurred in the called contract
    "#});
}

#[tokio::test]
async fn test_wrong_calldata() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        MAP_CONTRACT_ADDRESS,
        "--function-name",
        "get",
        "--calldata",
        "0x1 0x2",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stderr_matches(indoc! {r#"
        error: Execution was reverted; failure reason: [0x496e70757420746f6f206c6f6e6720666f7220617267756d656e7473].
    "#});
}
