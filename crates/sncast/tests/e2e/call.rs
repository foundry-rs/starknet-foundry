use crate::helpers::constants::{
    ACCOUNT_FILE_PATH, DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA, DATA_TRANSFORMER_CONTRACT_DIR,
    MAP_CONTRACT_ADDRESS_SEPOLIA, URL,
};
use crate::helpers::fixtures::{
    copy_directory_to_tempdir, create_and_deploy_oz_account, invoke_contract, join_tempdirs,
};
use crate::helpers::runner::runner;
use crate::helpers::shell::os_specific_shell;
use camino::Utf8PathBuf;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;
use snapbox::cargo_bin;
use sncast::helpers::fee::FeeSettings;

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
        Success: Call completed

        Response:     0x0
        Response Raw: [0x0]
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
        Success: Call completed
        
        Response: []
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
        tip: Some(100_000),
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
        Success: Call completed

        Response:     0x3
        Response Raw: [0x3]
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
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        r#"Error: Function with selector "0x2924aec1f107eca35a5dc447cee68cc6985fe404841c9aad477adfcbe596d0a" not found in ABI of the contract"#,
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
    let output = snapbox.assert().failure();

    // TODO(#3107)
    // 0x496e70757420746f6f206c6f6e6720666f7220617267756d656e7473 is "Input too long for arguments"
    assert_stderr_contains(
        output,
        indoc! {r#"
        Command: call
        Error: An error occurred in the called contract = [..] error: Message("[\"0x496e70757420746f6f206c6f6e6720666f7220617267756d656e7473\"]") }) }
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
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: call
        Error: Block was not found
        "},
    );
}

#[test]
fn test_happy_case_shell() {
    let binary_path = cargo_bin!("sncast");
    let command = os_specific_shell(&Utf8PathBuf::from("tests/shell/call"));

    let snapbox = command
        .arg(binary_path)
        .arg(URL)
        .arg(DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA);
    snapbox.assert().success();
}

#[test]
fn test_leading_negative_values() {
    let binary_path = cargo_bin!("sncast");
    let command = os_specific_shell(&Utf8PathBuf::from("tests/shell/call_unsigned"));

    let snapbox = command
        .arg(binary_path)
        .arg(URL)
        .arg(DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA);
    snapbox.assert().success();
}

#[test]
fn test_json_output_format() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--json",
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

    snapbox.assert().success().stdout_eq(indoc! {r#"
        {"command":"call","response":"0x0","response_raw":["0x0"],"type":"response"}
    "#});
}

async fn deploy_data_transformer_contract() -> (tempfile::TempDir, String) {
    let contract_dir = copy_directory_to_tempdir(DATA_TRANSFORMER_CONTRACT_DIR);
    let tempdir = create_and_deploy_oz_account().await;
    join_tempdirs(&contract_dir, &tempdir);

    let contents = std::fs::read_to_string(tempdir.path().join("accounts.json")).unwrap();
    let items: serde_json::Value = serde_json::from_str(&contents).unwrap();
    let account_address = items["alpha-sepolia"]["my_account"]["address"]
        .as_str()
        .unwrap()
        .to_string();

    let deploy_args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "--json",
        "deploy",
        "--url",
        URL,
        "--contract-name",
        "DataTransformer",
        "--arguments",
        &account_address,
    ];
    let deploy_output = runner(&deploy_args)
        .current_dir(tempdir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let contract_address = deploy_output
        .split(|&b| b == b'\n')
        .find_map(|line| {
            let json: serde_json::Value = serde_json::from_slice(line).ok()?;
            json["contract_address"].as_str().map(str::to_string)
        })
        .unwrap();

    (tempdir, contract_address)
}

#[tokio::test]
async fn test_happy_case_call_with_option() {
    let (tempdir, contract_address) = deploy_data_transformer_contract().await;

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "call",
        "--url",
        URL,
        "--contract-address",
        &contract_address,
        "--function",
        "option_fn",
        "--arguments",
        "Option::Some(42_u32)",
    ];

    runner(&args)
        .current_dir(tempdir.path())
        .assert()
        .success()
        .stdout_eq(indoc! {r"
            Success: Call completed

            Response: [0x0, 0x2a]
        "});
}

#[tokio::test]
async fn test_happy_case_call_with_result() {
    let (tempdir, contract_address) = deploy_data_transformer_contract().await;

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "call",
        "--url",
        URL,
        "--contract-address",
        &contract_address,
        "--function",
        "result_fn",
        "--arguments",
        "Result::Ok(99_u32)",
    ];

    runner(&args)
        .current_dir(tempdir.path())
        .assert()
        .success()
        .stdout_eq(indoc! {r"
            Success: Call completed

            Response: [0x0, 0x63]
        "});
}
