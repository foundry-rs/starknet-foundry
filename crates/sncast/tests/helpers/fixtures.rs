use crate::helpers::constants::{ACCOUNT_FILE_PATH, DEVNET_OZ_CLASS_HASH_CAIRO_0, URL};
use crate::helpers::runner::runner;
use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};
use conversions::string::IntoHexStr;
use core::str;
use fs_extra::dir::{CopyOptions, copy};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};
use sncast::helpers::account::load_accounts;
use sncast::helpers::braavos::BraavosAccountFactory;
use sncast::helpers::constants::{
    BRAAVOS_BASE_ACCOUNT_CLASS_HASH, BRAAVOS_CLASS_HASH, OZ_CLASS_HASH, READY_CLASS_HASH,
};
use sncast::helpers::fee::FeeSettings;
use sncast::helpers::scarb_utils::get_package_metadata;
use sncast::state::state_file::{
    ScriptTransactionEntry, ScriptTransactionOutput, ScriptTransactionStatus,
};
use sncast::{AccountType, apply_optional_fields, get_chain_id, get_keystore_password};
use sncast::{get_account, get_provider};
use starknet::accounts::{
    Account, AccountFactory, ArgentAccountFactory, ExecutionV3, OpenZeppelinAccountFactory,
};
use starknet::core::types::{Call, InvokeTransactionResult, TransactionReceipt};
use starknet::core::utils::get_contract_address;
use starknet::core::utils::get_selector_from_name;
use starknet::providers::JsonRpcClient;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::signers::{LocalWallet, SigningKey};
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufRead, Write};
use tempfile::{TempDir, tempdir};
use toml::Table;
use url::Url;

use super::fee::apply_test_resource_bounds_flags;

const SCRIPT_ORIGIN_TIMESTAMP: u64 = 1_709_853_748;

pub async fn deploy_keystore_account() {
    let keystore_path = "tests/data/keystore/predeployed_key.json";
    let account_path = "tests/data/keystore/predeployed_account.json";
    let private_key =
        SigningKey::from_keystore(keystore_path, "123").expect("Failed to get private_key");

    let contents = fs::read_to_string(account_path).expect("Failed to read keystore account file");
    let items: Value = serde_json::from_str(&contents)
        .unwrap_or_else(|_| panic!("Failed to parse keystore account file at = {account_path}"));

    let deployment_info = items
        .get("deployment")
        .expect("Failed to get deployment key");
    let address = get_from_json_as_str(deployment_info, "address");

    deploy_oz_account(
        address,
        DEVNET_OZ_CLASS_HASH_CAIRO_0,
        "0xa5d90c65b1b1339",
        private_key,
    )
    .await;
}

pub async fn deploy_cairo_0_account() {
    let (address, salt, private_key) = get_account_deployment_data("oz_cairo_0");
    deploy_oz_account(
        address.as_str(),
        DEVNET_OZ_CLASS_HASH_CAIRO_0,
        salt.as_str(),
        private_key,
    )
    .await;
}
pub async fn deploy_latest_oz_account() {
    let (address, salt, private_key) = get_account_deployment_data("oz");
    deploy_oz_account(
        address.as_str(),
        OZ_CLASS_HASH.into_hex_string().as_str(),
        salt.as_str(),
        private_key,
    )
    .await;
}
pub async fn deploy_ready_account() {
    let provider = get_provider(URL).expect("Failed to get the provider");
    let chain_id = get_chain_id(&provider)
        .await
        .expect("Failed to get chain id");

    let (address, salt, private_key) = get_account_deployment_data("ready");

    let factory = ArgentAccountFactory::new(
        READY_CLASS_HASH,
        chain_id,
        None,
        LocalWallet::from_signing_key(private_key),
        provider,
    )
    .await
    .expect("Failed to create Account Factory");

    deploy_account_to_devnet(factory, address.as_str(), salt.as_str()).await;
}

pub async fn deploy_braavos_account() {
    let provider = get_provider(URL).expect("Failed to get the provider");
    let chain_id = get_chain_id(&provider)
        .await
        .expect("Failed to get chain id");

    let (address, salt, private_key) = get_account_deployment_data("braavos");

    let factory = BraavosAccountFactory::new(
        BRAAVOS_CLASS_HASH,
        BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
        chain_id,
        LocalWallet::from_signing_key(private_key),
        provider,
    )
    .await
    .expect("Failed to create Account Factory");

    deploy_account_to_devnet(factory, address.as_str(), salt.as_str()).await;
}

async fn deploy_oz_account(address: &str, class_hash: &str, salt: &str, private_key: SigningKey) {
    let provider = get_provider(URL).expect("Failed to get the provider");
    let chain_id = get_chain_id(&provider)
        .await
        .expect("Failed to get chain id");

    let factory = OpenZeppelinAccountFactory::new(
        class_hash.parse().expect("Failed to parse class hash"),
        chain_id,
        LocalWallet::from_signing_key(private_key),
        provider,
    )
    .await
    .expect("Failed to create Account Factory");

    deploy_account_to_devnet(factory, address, salt).await;
}

async fn deploy_account_to_devnet<T: AccountFactory + Sync>(factory: T, address: &str, salt: &str) {
    mint_token(address, u128::MAX).await;
    factory
        .deploy_v3(salt.parse().expect("Failed to parse salt"))
        .l1_gas(100_000)
        .l1_gas_price(10_000_000_000_000)
        .l2_gas(1_000_000)
        .l2_gas_price(10_000_000_000_000)
        .l1_data_gas(100_000)
        .l1_data_gas_price(10_000_000_000_000)
        .send()
        .await
        .expect("Failed to deploy account");
}

fn get_account_deployment_data(account: &str) -> (String, String, SigningKey) {
    let items =
        load_accounts(&Utf8PathBuf::from(ACCOUNT_FILE_PATH)).expect("Failed to load accounts");

    let account_data = items
        .get("alpha-sepolia")
        .and_then(|accounts| accounts.get(account))
        .unwrap_or_else(|| panic!("Failed to get {account} account"));

    let address = get_from_json_as_str(account_data, "address");
    let salt = get_from_json_as_str(account_data, "salt");
    let private_key = get_from_json_as_str(account_data, "private_key");

    let private_key = SigningKey::from_secret_scalar(
        private_key
            .parse()
            .expect("Failed to convert private key to Felt"),
    );

    (address.to_string(), salt.to_string(), private_key)
}

fn get_from_json_as_str<'a>(entry: &'a Value, key: &str) -> &'a str {
    entry
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("Failed to get {key} key"))
}

pub async fn invoke_contract(
    account: &str,
    contract_address: &str,
    entry_point_name: &str,
    fee_settings: FeeSettings,
    constructor_calldata: &[&str],
) -> InvokeTransactionResult {
    let provider = get_provider(URL).expect("Could not get the provider");
    let account = get_account(
        account,
        &Utf8PathBuf::from(ACCOUNT_FILE_PATH),
        &provider,
        None,
    )
    .await
    .expect("Could not get the account");

    let mut calldata: Vec<Felt> = vec![];

    for value in constructor_calldata {
        let value: Felt = value.parse().expect("Could not parse the calldata");
        calldata.push(value);
    }

    let call = Call {
        to: contract_address
            .parse()
            .expect("Could not parse the contract address"),
        selector: get_selector_from_name(entry_point_name)
            .unwrap_or_else(|_| panic!("Could not get selector from {entry_point_name}")),
        calldata,
    };

    let execution = account.execute_v3(vec![call]);
    let execution = apply_optional_fields!(
        execution,
        fee_settings.l1_gas => ExecutionV3::l1_gas,
        fee_settings.l1_gas_price => ExecutionV3::l1_gas_price,
        fee_settings.l2_gas => ExecutionV3::l2_gas,
        fee_settings.l2_gas_price => ExecutionV3::l2_gas_price,
        fee_settings.l1_data_gas => ExecutionV3::l1_data_gas,
        fee_settings.l1_data_gas_price => ExecutionV3::l1_data_gas_price
    );

    execution
        .send()
        .await
        .expect("Transaction execution failed")
}

pub async fn mint_token(recipient: &str, amount: u128) {
    let client = reqwest::Client::new();
    let json = json!(
        {
            "address": recipient,
            "amount": amount,
            "unit": "FRI",
        }
    );
    client
        .post("http://127.0.0.1:5055/mint")
        .header("Content-Type", "application/json")
        .body(json.to_string())
        .send()
        .await
        .expect("Error occurred while minting tokens");
}

#[must_use]
pub fn default_cli_args() -> Vec<&'static str> {
    vec!["--url", URL, "--accounts-file", ACCOUNT_FILE_PATH]
}

fn parse_output<T: DeserializeOwned>(output: &[u8]) -> T {
    for line in BufRead::split(output, b'\n') {
        let line = line.expect("Failed to read line from stdout");
        if let Ok(t) = serde_json::de::from_slice::<T>(&line) {
            return t;
        }
    }

    panic!("Failed to deserialize stdout JSON to struct");
}

#[derive(Deserialize)]
#[expect(dead_code)]
struct TransactionHashOutput {
    pub transaction_hash: String,
    contract_address: Option<String>,
    class_hash: Option<String>,
    command: Option<String>,
}

#[must_use]
pub fn get_transaction_hash(output: &[u8]) -> Felt {
    let output = parse_output::<TransactionHashOutput>(output);
    output
        .transaction_hash
        .parse()
        .expect("Could not parse a number")
}

pub async fn get_transaction_receipt(tx_hash: Felt) -> TransactionReceipt {
    let client = reqwest::Client::new();
    let json = json!(
        {
            "jsonrpc": "2.0",
            "method": "starknet_getTransactionReceipt",
            "params": {
                "transaction_hash": format!("{tx_hash:#x}"),
            },
            "id": 0,
        }
    );
    let resp: Value = serde_json::from_str(
        &client
            .post(URL)
            .header("Content-Type", "application/json")
            .body(json.to_string())
            .send()
            .await
            .expect("Error occurred while getting transaction receipt")
            .text()
            .await
            .expect("Could not get response from getTransactionReceipt"),
    )
    .expect("Could not serialize getTransactionReceipt response");

    let result = resp
        .get("result")
        .expect("There is no `result` field in getTransactionReceipt response");
    serde_json::from_str(&result.to_string())
        .expect("Could not serialize result to `TransactionReceipt`")
}

#[must_use]
pub fn create_test_provider() -> JsonRpcClient<HttpTransport> {
    let parsed_url = Url::parse(URL).unwrap();
    JsonRpcClient::new(HttpTransport::new(parsed_url))
}

pub fn copy_file(src_path: impl AsRef<std::path::Path>, dest_path: impl AsRef<std::path::Path>) {
    fs_extra::file::copy(
        src_path.as_ref(),
        dest_path.as_ref(),
        &fs_extra::file::CopyOptions::new().overwrite(true),
    )
    .expect("Failed to copy the file");
}

#[must_use]
pub fn duplicate_contract_directory_with_salt(
    src_dir: impl AsRef<Utf8Path>,
    code_to_be_salted: &str,
    salt: &str,
) -> TempDir {
    let src_dir = Utf8PathBuf::from(src_dir.as_ref());

    let temp_dir = copy_directory_to_tempdir(&src_dir);

    let dest_dir = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .expect("Failed to create Utf8PathBuf from PathBuf");

    let file_to_be_salted = "src/lib.cairo";
    let contract_code =
        fs::read_to_string(src_dir.join(file_to_be_salted)).expect("Unable to get contract code");
    let updated_code =
        contract_code.replace(code_to_be_salted, &(code_to_be_salted.to_string() + salt));
    fs::write(dest_dir.join(file_to_be_salted), updated_code)
        .expect("Unable to change contract code");

    temp_dir
}

#[must_use]
pub fn copy_directory_to_tempdir(src_dir: impl AsRef<Utf8Path>) -> TempDir {
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");

    fs_extra::dir::copy(
        src_dir.as_ref(),
        temp_dir.as_ref(),
        &fs_extra::dir::CopyOptions::new()
            .overwrite(true)
            .content_only(true),
    )
    .expect("Failed to copy the directory");

    temp_dir
}

fn copy_script_directory(
    src_dir: impl AsRef<Utf8Path>,
    dest_dir: impl AsRef<Utf8Path>,
    deps: Vec<impl AsRef<std::path::Path>>,
) {
    let src_dir = Utf8PathBuf::from(src_dir.as_ref());
    let dest_dir = Utf8PathBuf::from(dest_dir.as_ref());
    let mut deps = get_deps_map_from_paths(deps);

    let manifest_path = dest_dir.join("Scarb.toml");
    let contents = fs::read_to_string(&manifest_path).unwrap();
    let mut parsed_toml: Table = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse {manifest_path}"))
        .unwrap();

    let deps_toml = parsed_toml
        .get_mut("dependencies")
        .unwrap()
        .as_table_mut()
        .unwrap();

    let sncast_std = deps_toml
        .get_mut("sncast_std")
        .expect("sncast_std not found");

    let sncast_std_path = sncast_std.get_mut("path").expect("No path to sncast_std");
    let sncast_std_path =
        Utf8PathBuf::from(sncast_std_path.as_str().expect("Failed to extract string"));

    let sncast_std_path = src_dir.join(sncast_std_path);
    let sncast_std_path_absolute = sncast_std_path
        .canonicalize_utf8()
        .expect("Failed to canonicalize sncast_std path");
    deps.insert(String::from("sncast_std"), sncast_std_path_absolute);

    for (key, value) in deps {
        let pkg = deps_toml.get_mut(&key).unwrap().as_table_mut().unwrap();
        pkg.insert("path".to_string(), toml::Value::String(value.to_string()));
    }

    let modified_toml = toml::to_string(&parsed_toml).expect("Failed to serialize TOML");

    let mut file = File::create(manifest_path).expect("Failed to create file");
    file.write_all(modified_toml.as_bytes())
        .expect("Failed to write to file");
}

pub fn copy_script_directory_to_tempdir(
    src_dir: impl AsRef<Utf8Path>,
    deps: Vec<impl AsRef<std::path::Path>>,
) -> TempDir {
    let temp_dir = copy_directory_to_tempdir(&src_dir);

    let dest_dir = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .expect("Failed to create Utf8PathBuf from PathBuf");

    copy_script_directory(&src_dir, dest_dir, deps);

    temp_dir
}

pub fn copy_workspace_directory_to_tempdir(
    src_dir: impl AsRef<Utf8Path>,
    relative_member_paths: Vec<impl AsRef<std::path::Path>>,
    deps: &[impl AsRef<std::path::Path> + Clone],
) -> TempDir {
    let src_dir = Utf8PathBuf::from(src_dir.as_ref());

    let temp_dir = copy_directory_to_tempdir(&src_dir);

    let dest_dir = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .expect("Failed to create Utf8PathBuf from PathBuf");

    for member in relative_member_paths {
        let member = member.as_ref().to_str().unwrap();
        let src_member_path = src_dir.join(member);
        let dest_member_path = dest_dir.join(member);
        fs::create_dir_all(&dest_member_path).expect("Failed to create directories in temp dir");
        copy_script_directory(&src_member_path, dest_member_path, deps.to_vec());
    }

    temp_dir
}

#[must_use]
pub fn get_deps_map_from_paths(
    paths: Vec<impl AsRef<std::path::Path>>,
) -> HashMap<String, Utf8PathBuf> {
    let mut deps = HashMap::<String, Utf8PathBuf>::new();

    for path in paths {
        let path = Utf8PathBuf::from_path_buf(path.as_ref().to_path_buf())
            .expect("Failed to create Utf8PathBuf from PathBuf");
        let manifest_path = path.join("Scarb.toml");
        let package =
            get_package_metadata(&manifest_path, &None).expect("Failed to get package metadata");
        deps.insert(package.name.clone(), path);
    }

    deps
}

pub fn get_address_from_keystore(
    keystore_path: impl AsRef<std::path::Path>,
    account_path: impl AsRef<std::path::Path>,
    password: &str,
    account_type: &AccountType,
) -> Felt {
    let contents = std::fs::read_to_string(account_path).unwrap();
    let items: Map<String, serde_json::Value> = serde_json::from_str(&contents).unwrap();
    let deployment = items.get("deployment").unwrap();

    let private_key = SigningKey::from_keystore(
        keystore_path,
        get_keystore_password(password).unwrap().as_str(),
    )
    .unwrap();
    let salt = Felt::from_hex(
        deployment
            .get("salt")
            .and_then(serde_json::Value::as_str)
            .unwrap(),
    )
    .unwrap();
    let class_hash = match account_type {
        AccountType::Braavos => BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
        AccountType::OpenZeppelin | AccountType::Argent | AccountType::Ready => Felt::from_hex(
            deployment
                .get("class_hash")
                .and_then(serde_json::Value::as_str)
                .unwrap(),
        )
        .unwrap(),
    };

    let calldata = match account_type {
        AccountType::OpenZeppelin | AccountType::Braavos => {
            vec![private_key.verifying_key().scalar()]
        }
        // This is a serialization of `Signer` enum for the variant `StarknetSigner` from the Ready account code
        // One stands for `None` for the guardian argument
        AccountType::Argent | AccountType::Ready => {
            vec![Felt::ZERO, private_key.verifying_key().scalar(), Felt::ONE]
        }
    };

    get_contract_address(salt, class_hash, &calldata, Felt::ZERO)
}
#[must_use]
pub fn get_accounts_path(relative_path_from_cargo_toml: &str) -> String {
    use std::path::PathBuf;
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let binding = PathBuf::from(manifest_dir).join(relative_path_from_cargo_toml);
    binding
        .to_str()
        .expect("Failed to convert path to string")
        .to_string()
}
#[must_use]
pub fn get_keystores_path(relative_path_from_cargo_toml: &str) -> String {
    use std::path::PathBuf;
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let binding = PathBuf::from(manifest_dir).join(relative_path_from_cargo_toml);
    binding
        .to_str()
        .expect("Failed to convert path to string")
        .to_string()
}

pub fn assert_tx_entry_failed(
    tx_entry: &ScriptTransactionEntry,
    name: &str,
    status: ScriptTransactionStatus,
    msg_contains: Vec<&str>,
) {
    assert_eq!(tx_entry.name, name);
    assert_eq!(tx_entry.status, status);

    let ScriptTransactionOutput::ErrorResponse(response) = &tx_entry.output else {
        panic!("Wrong response")
    };
    for msg in msg_contains {
        assert!(response.message.contains(msg));
    }

    assert!(tx_entry.timestamp > SCRIPT_ORIGIN_TIMESTAMP);
}

pub fn assert_tx_entry_success(tx_entry: &ScriptTransactionEntry, name: &str) {
    assert_eq!(tx_entry.name, name);
    assert_eq!(tx_entry.status, ScriptTransactionStatus::Success);

    let expected_selector = match tx_entry.output {
        ScriptTransactionOutput::DeployResponse(_) => "deploy",
        ScriptTransactionOutput::DeclareResponse(_) => "declare",
        ScriptTransactionOutput::InvokeResponse(_) => "invoke",
        ScriptTransactionOutput::ErrorResponse(_) => panic!("Error response received"),
    };
    assert_eq!(expected_selector, name);

    assert!(tx_entry.timestamp > SCRIPT_ORIGIN_TIMESTAMP);
}

pub async fn create_and_deploy_oz_account() -> TempDir {
    create_and_deploy_account(OZ_CLASS_HASH, AccountType::OpenZeppelin).await
}
pub async fn create_and_deploy_account(class_hash: Felt, account_type: AccountType) -> TempDir {
    let class_hash = &class_hash.into_hex_string();
    let account_type = match account_type {
        AccountType::OpenZeppelin => "oz",
        AccountType::Argent => "argent",
        AccountType::Ready => "ready",
        AccountType::Braavos => "braavos",
    };
    let tempdir = tempdir().unwrap();
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--url",
        URL,
        "--name",
        "my_account",
        "--class-hash",
        class_hash,
        "--type",
        account_type,
    ];

    runner(&args).current_dir(tempdir.path()).assert().success();

    let contents = fs::read_to_string(tempdir.path().join(accounts_file)).unwrap();
    let items: Value = serde_json::from_str(&contents).unwrap();

    mint_token(
        items["alpha-sepolia"]["my_account"]["address"]
            .as_str()
            .unwrap(),
        u128::MAX,
    )
    .await;

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--json",
        "account",
        "deploy",
        "--url",
        URL,
        "--name",
        "my_account",
    ];
    let args = apply_test_resource_bounds_flags(args);

    runner(&args).current_dir(tempdir.path()).assert().success();

    tempdir
}

pub fn join_tempdirs(from: &TempDir, to: &TempDir) {
    copy(
        from.path(),
        to.path(),
        &CopyOptions::new().overwrite(true).content_only(true),
    )
    .expect("Failed to copy the directory");
}
