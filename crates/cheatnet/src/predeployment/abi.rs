use crate::predeployment::erc20::eth::ERC20MINTABLE_SIERRA_CLASS_HASH;
use crate::predeployment::erc20::strk::ERC20LOCKABLE_SIERRA_CLASS_HASH;
use crate::runtime_extensions::forge_runtime_extension::contracts_data::build_name_selector_map;
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

fn abi_entry(class_hash_str: &str, abi_json: &str) -> (ClassHash, Vec<AbiEntry>) {
    let class_hash = parse_class_hash(class_hash_str);
    let abi = serde_json::from_str::<Vec<AbiEntry>>(abi_json).expect("ABI should be valid");
    (class_hash, abi)
}

fn parse_class_hash(class_hash_str: &str) -> ClassHash {
    TryFromHexStr::try_from_hex_str(class_hash_str).expect("class hash should be valid")
}

#[derive(Clone)]
pub struct PredeployedContractsDebuggingData {
    pub abi: HashMap<ClassHash, Vec<AbiEntry>>,
    pub selectors: HashMap<EntryPointSelector, String>,
    pub contract_names: HashMap<ClassHash, String>,
}

fn build_predeployed_contracts_debugging_data() -> PredeployedContractsDebuggingData {
    let abi = HashMap::from([
        abi_entry(ERC20LOCKABLE_SIERRA_CLASS_HASH, ERC20LOCKABLE_ABI_JSON),
        abi_entry(ERC20MINTABLE_SIERRA_CLASS_HASH, ERC20MINTABLE_ABI_JSON),
    ]);

    let selectors = abi
        .values()
        .flat_map(|a| build_name_selector_map(a.clone()))
        .collect();

    let contract_names = HashMap::from([
        (
            parse_class_hash(ERC20LOCKABLE_SIERRA_CLASS_HASH),
            "STRK (predeployed)".to_string(),
        ),
        (
            parse_class_hash(ERC20MINTABLE_SIERRA_CLASS_HASH),
            "ETH (predeployed)".to_string(),
        ),
    ]);

    PredeployedContractsDebuggingData {
        abi,
        selectors,
        contract_names,
    }
}

static PREDEPLOYED_CONTRACTS_DEBUGGING_DATA: LazyLock<PredeployedContractsDebuggingData> =
    LazyLock::new(build_predeployed_contracts_debugging_data);

/// Returns debugging data for contracts predeployed in every `snforge` test environment.
#[must_use]
pub fn predeployed_contracts_debugging_data() -> PredeployedContractsDebuggingData {
    PREDEPLOYED_CONTRACTS_DEBUGGING_DATA.clone()
}
