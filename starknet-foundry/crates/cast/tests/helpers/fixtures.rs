use crate::helpers::constants::{
    ACCOUNT, ACCOUNT_FILE_PATH, CONTRACTS_DIR, MAP_CONTRACT_ADDRESS, NETWORK, URL,
};
use anyhow::Result;
use camino::Utf8PathBuf;
use cast::{get_account, get_network, get_provider, parse_number};
use serde_json::{json, Value};
use starknet::accounts::{Account, Call};
use starknet::contract::ContractFactory;
use starknet::core::types::contract::{CompiledClass, SierraClass};
use starknet::core::types::FieldElement;
use starknet::core::types::TransactionReceipt;
use starknet::core::utils::get_selector_from_name;
use std::collections::HashMap;
use std::sync::Arc;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use url::Url;

pub async fn declare_deploy_simple_balance_contract() {
    let network = get_network(&NETWORK).unwrap();
    let provider = get_provider(URL, &network).await.expect("Could not get the provider");
    let account = get_account(
        ACCOUNT,
        &Utf8PathBuf::from(ACCOUNT_FILE_PATH),
        &provider,
        &get_network(NETWORK).expect("Could not get the network"),
    )
    .expect("Could not get the account");

    let contract_definition: SierraClass = {
        let file_contents =
            std::fs::read(CONTRACTS_DIR.to_string() + "/map/target/dev/map_Map.sierra.json")
                .expect("Could not read balance's sierra file");
        serde_json::from_slice(&file_contents).expect("Could not cast sierra file to SierraClass")
    };
    let casm_contract_definition: CompiledClass = {
        let file_contents =
            std::fs::read(CONTRACTS_DIR.to_string() + "/map/target/dev/map_Map.casm.json")
                .expect("Could not read balance's casm file");
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
    let declared = declaration.send().await.unwrap();

    let factory = ContractFactory::new(declared.class_hash, account);
    let deployment = factory.deploy(Vec::new(), FieldElement::ONE, false);
    deployment.send().await.unwrap();
}

pub async fn invoke_map_contract(key: &str, value: &str) {
    let network = get_network(&NETWORK).unwrap();
    let provider = get_provider(URL, &network).await.expect("Could not get the provider");
    let account = get_account(
        ACCOUNT,
        &Utf8PathBuf::from(ACCOUNT_FILE_PATH),
        &provider,
        &get_network(NETWORK).expect("Could not get the network"),
    )
    .expect("Could not get the account");

    let call = Call {
        to: parse_number(MAP_CONTRACT_ADDRESS).expect("Could not parse the contract address"),
        selector: get_selector_from_name("put").expect("Could not get selector from put"),
        calldata: vec![
            parse_number(key).expect("Could not parse the key"),
            parse_number(value).expect("Could not parse the value"),
        ],
    };
    let execution = account.execute(vec![call]);

    execution.send().await.unwrap();
}

#[must_use]
pub fn default_cli_args() -> Vec<&'static str> {
    vec![
        "--url",
        URL,
        "--network",
        NETWORK,
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        ACCOUNT,
    ]
}

#[must_use]
pub fn get_transaction_hash(output: &[u8]) -> FieldElement {
    let output: HashMap<String, String> =
        serde_json::from_slice(output).expect("Could not serialize transaction output to HashMap");
    parse_number(
        output
            .get("transaction_hash")
            .expect("Could not get transaction_hash from output"),
    )
    .expect("Could not parse a number")
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

pub fn create_test_provider() -> JsonRpcClient<HttpTransport> {
    let parsed_url = Url::parse(URL).unwrap();
    let provider = JsonRpcClient::new(HttpTransport::new(parsed_url));
    return provider
}
