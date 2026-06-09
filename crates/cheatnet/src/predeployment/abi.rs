use crate::predeployment::erc20::eth::ERC20MINTABLE_SIERRA_CLASS_HASH;
use crate::predeployment::erc20::strk::ERC20LOCKABLE_SIERRA_CLASS_HASH;
use crate::runtime_extensions::forge_runtime_extension::contracts_data::build_selectors_from_abi_map;
use conversions::string::TryFromHexStr;
use starknet_api::core::{ClassHash, EntryPointSelector};
use starknet_rust::core::types::contract::AbiEntry;
use std::collections::HashMap;
use std::sync::LazyLock;

// starkgate-contracts v3.0.0
const ERC20LOCKABLE_ABI_JSON: &str =
    include_str!("../data/predeployed_contracts/ERC20Lockable/abi.json");
const ERC20MINTABLE_ABI_JSON: &str =
    include_str!("../data/predeployed_contracts/ERC20Mintable/abi.json");

#[derive(Clone)]
pub struct ContractsDebuggingData {
    pub abi: HashMap<ClassHash, Vec<AbiEntry>>,
    pub selectors: HashMap<EntryPointSelector, String>,
    pub contract_names: HashMap<ClassHash, String>,
}

fn parse_class_hash(class_hash_str: &str) -> ClassHash {
    TryFromHexStr::try_from_hex_str(class_hash_str).expect("class hash should be valid")
}

fn parse_abi(abi_json: &str) -> Vec<AbiEntry> {
    serde_json::from_str::<Vec<AbiEntry>>(abi_json).expect("ABI should be valid")
}

fn build_predeployed_contracts_debugging_data() -> ContractsDebuggingData {
    let strk_class_hash = parse_class_hash(ERC20LOCKABLE_SIERRA_CLASS_HASH);
    let eth_class_hash = parse_class_hash(ERC20MINTABLE_SIERRA_CLASS_HASH);

    let strk_abi = parse_abi(ERC20LOCKABLE_ABI_JSON);
    let eth_abi = parse_abi(ERC20MINTABLE_ABI_JSON);

    let abi = HashMap::from([(strk_class_hash, strk_abi), (eth_class_hash, eth_abi)]);

    let selectors = build_selectors_from_abi_map(&abi);

    let contract_names = HashMap::from([
        (strk_class_hash, "STRK (predeployed)".to_string()),
        (eth_class_hash, "ETH (predeployed)".to_string()),
    ]);

    ContractsDebuggingData {
        abi,
        selectors,
        contract_names,
    }
}

static PREDEPLOYED_CONTRACTS_DEBUGGING_DATA: LazyLock<ContractsDebuggingData> =
    LazyLock::new(build_predeployed_contracts_debugging_data);

/// Returns debugging data for contracts predeployed in every `snforge` test environment.
#[must_use]
pub fn predeployed_contracts_debugging_data() -> ContractsDebuggingData {
    PREDEPLOYED_CONTRACTS_DEBUGGING_DATA.clone()
}
