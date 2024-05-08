use crate::helpers::constants::MAP_CONTRACT_ADDRESS_SEPOLIA;
use crate::helpers::fixtures::{default_cli_args, invoke_contract};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;

#[test]
fn test_happy_case() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
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
    invoke_contract(
        "user2",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "put",
        None,
        &["0x2", "0x3"],
    )
    .await;
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
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
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: call
        error: There is no contract at the specified address
        "},
    );
}

#[test]
fn test_wrong_function_name() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "nonexistent_get",
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: call
        error: An error occurred [..]Entry point[..]not found in contract[..]
        "},
    );
}

#[test]
fn test_wrong_calldata() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--calldata",
        "0x1",
        "0x2",
        "--function",
        "get",
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: call
        error: An error occurred [..]Execution failed[..]Input too long for arguments[..]
        "},
    );
}

#[tokio::test]
async fn test_invalid_selector() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "ą",
        "--calldata",
        "0x1 0x2",
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Error: Failed to convert entry point selector to FieldElement
    
        Caused by:
            the provided name contains non-ASCII characters
  "},
    );
}

#[test]
fn test_wrong_block_id() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "call",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "get",
        "--calldata",
        "0x0",
        "--block-id",
        "0x10101",
    ]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: call
        error: Block was not found
        "},
    );
}
