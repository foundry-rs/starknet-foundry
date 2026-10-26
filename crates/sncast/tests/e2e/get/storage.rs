use crate::helpers::constants::{
    ACCOUNT_FILE_PATH, CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA,
    MAP_CONTRACT_ADDRESS_SEPOLIA, URL,
};
use crate::helpers::fixtures::invoke_contract;
use crate::helpers::runner::runner;
use configuration::test_utils::copy_config_to_tempdir;
use indoc::indoc;
use serde_json::json;
use shared::test_utils::output_assert::assert_stdout_contains;
use sncast::helpers::fee::FeeSettings;
use starknet_rust::core::utils::{get_selector_from_name, get_storage_var_address};

fn map_storage_key(key: &str) -> String {
    get_storage_var_address("storage", &[key.parse().unwrap()])
        .unwrap()
        .to_hex_string()
}

async fn store_map_value(account: &str) {
    invoke_contract(
        account,
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "put",
        FeeSettings {
            l1_gas: Some(100_000),
            l1_gas_price: Some(10_000_000_000_000),
            l2_gas: Some(1_000_000_000),
            l2_gas_price: Some(100_000_000_000_000_000),
            l1_data_gas: Some(100_000),
            l1_data_gas_price: Some(10_000_000_000_000),
            tip: Some(100_000),
        },
        &["0x43", "0x2a"],
    )
    .await;
}

fn deploy_contract_with_value1(account: &str) -> String {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        account,
        "--json",
        "deploy",
        "--url",
        URL,
        "--class-hash",
        CONSTRUCTOR_WITH_PARAMS_CONTRACT_CLASS_HASH_SEPOLIA,
        "--constructor-calldata",
        "0x859 0x0 0x0",
        "--salt",
        "0x4242",
        "--unique",
    ];

    let output = runner(&args).assert().success();
    output
        .get_output()
        .stdout
        .split(|byte| *byte == b'\n')
        .filter_map(|line| serde_json::from_slice::<serde_json::Value>(line).ok())
        .find_map(|res| res["contract_address"].as_str().map(str::to_owned))
        .expect("Failed to deserialize contract address from deploy output")
}

#[tokio::test]
async fn happy_case_raw_with_last_update_block() {
    store_map_value("user13").await;
    let key = map_storage_key("0x43");
    let args = vec![
        "get",
        "storage",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--felt-key",
        &key,
        "--url",
        URL,
        "-l",
    ];

    let output = runner(&args).assert().success();
    assert_stdout_contains(output, "Last update block: [..]");
}

#[tokio::test]
async fn happy_case_raw() {
    store_map_value("user14").await;
    let key = map_storage_key("0x43");
    let args = vec![
        "get",
        "storage",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--felt-key",
        &key,
        "--url",
        URL,
    ];

    runner(&args).assert().success().stdout_eq(indoc! {r"
        Success: Value retrieved

        Key:             [..]
        Storage address: [..]
        Value:           0x2a
    "});
}

#[tokio::test]
async fn happy_case_json_raw() {
    store_map_value("user15").await;
    let key = map_storage_key("0x43");
    let args = vec![
        "get",
        "storage",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--felt-key",
        &key,
        "--url",
        URL,
        "--json",
    ];

    let output = runner(&args).assert().success();
    let stdout = output.get_output().stdout.clone();
    let res: serde_json::Value = serde_json::from_slice(&stdout).unwrap();

    assert_eq!(
        res,
        json!({
            "command": "get storage",
            "key": key,
            "storage_address": key,
            "type": "response",
            "value": "0x2a",
        })
    );
}

#[tokio::test]
async fn missing_key_returns_zero_raw() {
    let key = map_storage_key("0x44");
    let args = vec![
        "get",
        "storage",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--felt-key",
        &key,
        "--url",
        URL,
    ];

    runner(&args).assert().success().stdout_eq(indoc! {r"
        Success: Value retrieved

        Key:             [..]
        Storage address: [..]
        Value:           0x0
    "});
}

#[test]
fn happy_case_plaintext_key() {
    let contract_address = deploy_contract_with_value1("user16");
    let args = vec![
        "get",
        "storage",
        "--contract-address",
        &contract_address,
        "--key",
        "value1",
        "--url",
        URL,
    ];

    runner(&args).assert().success().stdout_eq(indoc! {r"
        Success: Value retrieved

        Key:             value1
        Storage address: [..]
        Value:           0x859
    "});
}

#[test]
fn happy_case_json_plaintext_key() {
    let contract_address = deploy_contract_with_value1("user17");
    let args = vec![
        "get",
        "storage",
        "--contract-address",
        &contract_address,
        "--key",
        "value1",
        "--url",
        URL,
        "--json",
    ];

    let output = runner(&args).assert().success();
    let stdout = output.get_output().stdout.clone();
    let res: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    let storage_address = get_selector_from_name("value1").unwrap().to_hex_string();

    assert_eq!(
        res,
        json!({
            "command": "get storage",
            "key": "value1",
            "storage_address": storage_address,
            "type": "response",
            "value": "0x859",
        })
    );
}

#[test]
fn missing_plaintext_key_returns_zero() {
    let contract_address = deploy_contract_with_value1("user18");
    let args = vec![
        "get",
        "storage",
        "--contract-address",
        &contract_address,
        "--key",
        "missing",
        "--url",
        URL,
    ];

    runner(&args).assert().success().stdout_eq(indoc! {r"
        Success: Value retrieved

        Key:             missing
        Storage address: [..]
        Value:           0x0
    "});
}

#[test]
fn test_with_alias() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    let args = vec![
        "get",
        "storage",
        "--contract-address",
        "@map",
        "--key",
        "foo",
        "--url",
        URL,
    ];

    runner(&args)
        .current_dir(tempdir.path())
        .assert()
        .success()
        .stdout_eq(indoc! {r"
        Success: Value retrieved

        Key:             foo
        Storage address: [..]
        Value:           0x0
    "});
}

#[test]
fn test_with_unknown_alias() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    let args = vec![
        "get",
        "storage",
        "--contract-address",
        "@unknown",
        "--key",
        "foo",
        "--url",
        URL,
    ];

    runner(&args)
        .current_dir(tempdir.path())
        .assert()
        .failure()
        .stderr_eq(indoc! {r"
            Command: get storage
            Error: Invalid contract address

            Caused by:
                Alias `unknown` not found in config
        "});
}
