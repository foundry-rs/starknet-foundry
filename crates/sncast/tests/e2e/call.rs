use crate::helpers::constants::{
    ACCOUNT_FILE_PATH, DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA, MAP_CONTRACT_ADDRESS_SEPOLIA, URL,
};
use crate::helpers::fixtures::invoke_contract;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;
use snapbox::cmd::{Command, cargo_bin};
use sncast::helpers::fee::FeeSettings;
use std::path::PathBuf;

#[test]
fn test_happy_case() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "call",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "get",
        "--calldata",
        "0x0",
        "--block-id",
        "latest",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        command: call
        response: [0x0]
    "});
}

#[test]
fn test_happy_case_cairo_expression_calldata() {
    let args = vec![
        "call",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--arguments",
        "0x0_felt252, 0x2137",
        "--block-id",
        "latest",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        command: call
        response: []
    "});
}

#[tokio::test]
async fn test_call_after_storage_changed() {
    let fee_settings = FeeSettings {
        l1_gas: Some(100_000),
        l1_gas_price: Some(10_000_000_000_000),
        l2_gas: Some(1_000_000_000),
        l2_gas_price: Some(100_000_000_000_000_000),
        l1_data_gas: Some(100_000),
        l1_data_gas_price: Some(10_000_000_000_000),
    };
    invoke_contract(
        "user2",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "put",
        fee_settings,
        &["0x2", "0x3"],
    )
    .await;

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "call",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "get",
        "--calldata",
        "0x2",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        command: call
        response: [0x3]
    "});
}

#[tokio::test]
async fn test_contract_does_not_exist() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "call",
        "--url",
        URL,
        "--contract-address",
        "0x1",
        "--function",
        "get",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: An error occurred in the called contract[..]Requested contract address[..]is not deployed[..]",
    );
}

#[test]
fn test_wrong_function_name() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "call",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "nonexistent_get",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: call
        error: Requested entrypoint does not exist in the contract
        "},
    );
}

#[test]
fn test_wrong_calldata() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "call",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--calldata",
        "0x1",
        "0x2",
        "--function",
        "get",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    // TODO(#3107)
    // 0x496e70757420746f6f206c6f6e6720666f7220617267756d656e7473 is "Input too long for arguments"
    assert_stderr_contains(
        output,
        indoc! {r#"
        command: call
        error: An error occurred in the called contract = [..] error: Message("[\"0x496e70757420746f6f206c6f6e6720666f7220617267756d656e7473\"]") }) }
        "#},
    );
}

#[tokio::test]
async fn test_invalid_selector() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "call",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "ą",
        "--calldata",
        "0x1 0x2",
    ];

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
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "call",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "get",
        "--calldata",
        "0x0",
        "--block-id",
        "0x10101",
    ];

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

#[test]
fn test_happy_case_shell() {
    let script_extension = if cfg!(windows) { ".ps1" } else { ".sh" };
    let test_path = PathBuf::from(format!("tests/shell/call{script_extension}"))
        .canonicalize()
        .unwrap();
    let binary_path = cargo_bin!("sncast");

    let command = if cfg!(windows) {
        Command::new("powershell")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(test_path)
    } else {
        Command::new(test_path)
    };

    let snapbox = command
        .arg(binary_path)
        .arg(URL)
        .arg(DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA);
    snapbox.assert().success();
}
