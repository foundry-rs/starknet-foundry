use crate::helpers::constants::{
    ACCOUNT_FILE_PATH, CONTRACTS_DIR, DEVNET_ENV_FILE, DEVNET_OZ_CLASS_HASH, URL,
};
use camino::Utf8PathBuf;
use primitive_types::U256;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sncast::{apply_optional, get_chain_id, get_keystore_password};
use sncast::{get_account, get_provider, parse_number};
use starknet::accounts::{Account, AccountFactory, Call, Execution, OpenZeppelinAccountFactory};
use starknet::contract::ContractFactory;
use starknet::core::types::contract::{CompiledClass, SierraClass};
use starknet::core::types::TransactionReceipt;
use starknet::core::types::{FieldElement, InvokeTransactionResult};
use starknet::core::utils::get_contract_address;
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::{LocalWallet, SigningKey};
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufRead, Write};
use std::sync::Arc;
use tempfile::TempDir;
use url::Url;

pub async fn declare_contract(account: &str, path: &str, shortname: &str) -> FieldElement {
    let provider = get_provider(URL).expect("Could not get the provider");
    let account = get_account(
        account,
        &Utf8PathBuf::from(ACCOUNT_FILE_PATH),
        &provider,
        None,
    )
    .await
    .expect("Could not get the account");

    let contract_definition: SierraClass = {
        let file_contents =
            std::fs::read(CONTRACTS_DIR.to_string() + path + ".contract_class.json")
                .expect("Could not read contract's sierra file");
        serde_json::from_slice(&file_contents).expect("Could not cast sierra file to SierraClass")
    };
    let casm_contract_definition: CompiledClass = {
        let file_contents =
            std::fs::read(CONTRACTS_DIR.to_string() + path + ".compiled_contract_class.json")
                .expect("Could not read contract's casm file");
        serde_json::from_slice(&file_contents).expect("Could not cast casm file to CompiledClass")
    };

    let casm_class_hash = casm_contract_definition
        .class_hash()
        .expect("Could not compute class_hash");

    let declaration = account.declare(
        Arc::new(
            contract_definition
                .flatten()
                .expect("Could not flatten SierraClass"),
        ),
        casm_class_hash,
    );

    let tx = declaration.send().await.unwrap();
    let class_hash = tx.class_hash;
    let tx_hash = tx.transaction_hash;
    write_devnet_env(format!("{shortname}_CLASS_HASH").as_str(), &class_hash);
    write_devnet_env(format!("{shortname}_DECLARE_HASH").as_str(), &tx_hash);
    class_hash
}

pub async fn deploy_keystore_account() {
    let keystore_path = "tests/data/keystore/predeployed_key.json";
    let account_path = "tests/data/keystore/predeployed_account.json";
    let private_key =
        SigningKey::from_keystore(keystore_path, "123").expect("Could not get the private_key");

    let provider = get_provider(URL).expect("Could not get the provider");
    let chain_id = get_chain_id(&provider)
        .await
        .expect("Could not get chain_id from provider");

    let contents =
        std::fs::read_to_string(account_path).expect("Failed to read keystore account file");
    let items: serde_json::Value = serde_json::from_str(&contents)
        .unwrap_or_else(|_| panic!("Failed to parse keystore account file at = {account_path}"));

    let factory = OpenZeppelinAccountFactory::new(
        parse_number(DEVNET_OZ_CLASS_HASH).expect("Could not parse DEVNET_OZ_CLASS_HASH"),
        chain_id,
        LocalWallet::from_signing_key(private_key),
        provider,
    )
    .await
    .expect("Could not create Account Factory");

    mint_token(
        items["deployment"]["address"]
            .as_str()
            .expect("Could not get address"),
        9_999_999_999_999_999_999,
    )
    .await;

    factory
        .deploy(parse_number("0xa5d90c65b1b1339").expect("Could not parse salt"))
        .send()
        .await
        .expect("Could not deploy keystore account");
}

pub async fn declare_deploy_contract(account: &str, path: &str, shortname: &str) {
    let class_hash = declare_contract(account, path, shortname).await;

    let provider = get_provider(URL).expect("Could not get the provider");
    let account = get_account(
        account,
        &Utf8PathBuf::from(ACCOUNT_FILE_PATH),
        &provider,
        None,
    )
    .await
    .expect("Could not get the account");

    let factory = ContractFactory::new(class_hash, &account);
    let deployment = factory.deploy(Vec::new(), FieldElement::ONE, true);

    let transaction_hash = deployment.send().await.unwrap().transaction_hash;
    let receipt = get_transaction_receipt(transaction_hash).await;
    match receipt {
        TransactionReceipt::Deploy(deploy_receipt) => {
            let address = deploy_receipt.contract_address;
            write_devnet_env(format!("{shortname}_ADDRESS").as_str(), &address);
        }
        _ => {
            panic!("Unexpected TransactionReceipt variant");
        }
    };
}

pub async fn invoke_contract(
    account: &str,
    contract_address: &str,
    entry_point_name: &str,
    max_fee: Option<FieldElement>,
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

    let mut calldata: Vec<FieldElement> = vec![];

    for value in constructor_calldata {
        let value: FieldElement = parse_number(value).expect("Could not parse the calldata");
        calldata.push(value);
    }

    let call = Call {
        to: parse_number(contract_address).expect("Could not parse the contract address"),
        selector: get_selector_from_name(entry_point_name)
            .unwrap_or_else(|_| panic!("Could not get selector from {entry_point_name}")),
        calldata,
    };

    let execution = account.execute(vec![call]);
    let execution = apply_optional(execution, max_fee, Execution::max_fee);

    execution.send().await.unwrap()
}

// devnet-rs accepts an amount as u128
// but serde_json cannot serialize numbers this big
pub async fn mint_token(recipient: &str, amount: u64) {
    let client = reqwest::Client::new();
    let json = json!(
        {
            "address": recipient,
            "amount": amount
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
        match serde_json::de::from_slice::<T>(&line) {
            Ok(t) => return t,
            Err(_) => continue,
        }
    }
    panic!("Failed to deserialize stdout JSON to struct");
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct TransactionHashOutput {
    pub transaction_hash: String,
    contract_address: Option<String>,
    class_hash: Option<String>,
    command: Option<String>,
}

#[must_use]
pub fn get_transaction_hash(output: &[u8]) -> FieldElement {
    let output = parse_output::<TransactionHashOutput>(output);
    parse_number(output.transaction_hash.as_str()).expect("Could not parse a number")
}

pub async fn get_transaction_receipt(tx_hash: FieldElement) -> TransactionReceipt {
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

#[must_use]
pub fn duplicate_directory_with_salt(src_path: String, to_be_salted: &str, salt: &str) -> TempDir {
    let src_dir = Utf8PathBuf::from(src_path);
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");
    let dest_dir = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf())
        .expect("Failed to create Utf8PathBuf from PathBuf");

    fs_extra::dir::copy(
        src_dir.join("src"),
        &dest_dir,
        &fs_extra::dir::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy the src directory");
    fs_extra::file::copy(
        src_dir.join("Scarb.toml"),
        dest_dir.join("Scarb.toml"),
        &fs_extra::file::CopyOptions::new().overwrite(true),
    )
    .expect("Unable to copy Scarb.toml");

    let contract_code =
        fs::read_to_string(src_dir.join("src/lib.cairo")).expect("Unable to get contract code");
    let updated_code = contract_code.replace(to_be_salted, &(to_be_salted.to_string() + salt));
    fs::write(dest_dir.join("src/lib.cairo"), updated_code)
        .expect("Unable to change contract code");

    temp_dir
}

pub fn remove_devnet_env() {
    if Utf8PathBuf::from(DEVNET_ENV_FILE).is_file() {
        fs::remove_file(DEVNET_ENV_FILE).unwrap();
    }
}

fn write_devnet_env(key: &str, value: &FieldElement) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(DEVNET_ENV_FILE)
        .unwrap();

    writeln!(file, "{key}={value}").unwrap();
}

#[must_use]
pub fn convert_to_hex(value: &str) -> String {
    let dec = U256::from_dec_str(value).expect("Invalid decimal string");
    format!("{dec:#x}")
}

pub fn from_env(name: &str) -> Result<String, String> {
    match env::var(name) {
        Ok(value) => Ok(value),
        Err(_) => Err(format!("Variable {name} not available in env!")),
    }
}

pub fn get_address_from_keystore(
    keystore_path: &str,
    account_path: &str,
    password: &str,
) -> FieldElement {
    let contents = std::fs::read_to_string(account_path).unwrap();
    let items: Map<String, serde_json::Value> = serde_json::from_str(&contents).unwrap();
    let deployment = items.get("deployment").unwrap();

    let private_key = SigningKey::from_keystore(
        keystore_path,
        get_keystore_password(password).unwrap().as_str(),
    )
    .unwrap();
    let salt = FieldElement::from_hex_be(
        deployment
            .get("salt")
            .and_then(serde_json::Value::as_str)
            .unwrap(),
    )
    .unwrap();
    let oz_class_hash = FieldElement::from_hex_be(
        deployment
            .get("class_hash")
            .and_then(serde_json::Value::as_str)
            .unwrap(),
    )
    .unwrap();

    get_contract_address(
        salt,
        oz_class_hash,
        &[private_key.verifying_key().scalar()],
        FieldElement::ZERO,
    )
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
