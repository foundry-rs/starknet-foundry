use crate::helpers::constants::{
    ACCOUNT, ACCOUNT_FILE_PATH, CONTRACTS_DIR, MAP_CONTRACT_ADDRESS, NETWORK, URL,
};
use camino::Utf8PathBuf;
use cast::{get_account, get_network, get_provider, parse_number};
use starknet::accounts::{Account, Call};
use starknet::contract::ContractFactory;
use starknet::core::types::contract::{CompiledClass, SierraClass};
use starknet::core::types::FieldElement;
use starknet::core::utils::get_selector_from_name;
use std::sync::Arc;

pub async fn declare_deploy_simple_balance_contract() {
    let provider = get_provider(URL).expect("Could not get the provider");
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
    let provider = get_provider(URL).expect("Could not get the provider");
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
