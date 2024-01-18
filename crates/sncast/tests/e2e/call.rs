use crate::helpers::fixtures::{default_cli_args, from_env, invoke_contract};
use crate::helpers::runner::runner;
use indoc::indoc;

#[test]
fn test_happy_case() {
    let contract_address = from_env("CAST_MAP_ADDRESS").unwrap();
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        &contract_address,
        "--function",
        "get",
        "--calldata",
        "0x0",
        "--block-id",
        "latest",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        command: call
        response: [0x0]
    "});
}

#[tokio::test]
async fn test_call_after_storage_changed() {
    let contract_address = from_env("CAST_MAP_ADDRESS").unwrap();
    invoke_contract("user2", &contract_address, "put", None, &["0x2", "0x3"]).await;
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        &contract_address,
        "--function",
        "get",
        "--calldata",
        "0x2",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        command: call
        response: [0x3]
    "});
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

    snapbox.assert().stderr_matches(indoc! {r"
        command: call
        error: There is no contract at the specified address
    "});
}

#[test]
fn test_wrong_function_name() {
    let contract_address = from_env("CAST_MAP_ADDRESS").unwrap();
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        &contract_address,
        "--function",
        "nonexistent_get",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r"
        command: call
        error: An error occurred in the called contract [..]
    "});
}

#[test]
fn test_wrong_calldata() {
    let contract_address = from_env("CAST_MAP_ADDRESS").unwrap();
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        &contract_address,
        "--calldata",
        "0x1",
        "0x2",
        "--function",
        "get",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r"
        command: call
        error: An error occurred in the called contract [..]
    "});
}

#[tokio::test]
async fn test_invalid_selector() {
    let address = from_env("CAST_MAP_ADDRESS").unwrap();
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        &address,
        "--function",
        "ą",
        "--calldata",
        "0x1 0x2",
    ]);

    let snapbox = runner(&args);
    snapbox.assert().stderr_matches(indoc! {r"
      command: call
      error: Failed to convert entry point selector to FieldElement: the provided name contains non-ASCII characters
  "});
}
