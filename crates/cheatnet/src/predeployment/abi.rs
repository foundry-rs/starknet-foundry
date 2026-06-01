use crate::forking::data::ForkData;
use crate::predeployment::erc20::eth::ERC20MINTABLE_SIERRA_CLASS_HASH;
use crate::predeployment::erc20::strk::ERC20LOCKABLE_SIERRA_CLASS_HASH;
use crate::runtime_extensions::forge_runtime_extension::contracts_data::build_name_selector_map;
use conversions::string::TryFromHexStr;
use starknet_api::core::ClassHash;
use starknet_rust::core::types::contract::AbiEntry;
use std::collections::HashMap;

const ERC20LOCKABLE_ABI_JSON: &str =
    include_str!("../data/predeployed_contracts/ERC20Lockable/abi.json");
const ERC20MINTABLE_ABI_JSON: &str =
    include_str!("../data/predeployed_contracts/ERC20Mintable/abi.json");

fn abi_entry(class_hash_str: &str, abi_json: &str) -> (ClassHash, Vec<AbiEntry>) {
    let class_hash: ClassHash =
        TryFromHexStr::try_from_hex_str(class_hash_str).expect("class hash should be valid");
    let abi = serde_json::from_str::<Vec<AbiEntry>>(abi_json).expect("ABI should be valid");
    (class_hash, abi)
}

/// Returns hardcoded ABI data for contracts predeployed in every snforge test environment.
#[must_use]
pub fn predeployed_contracts_fork_data() -> ForkData {
    let abi = HashMap::from([
        abi_entry(ERC20LOCKABLE_SIERRA_CLASS_HASH, ERC20LOCKABLE_ABI_JSON),
        abi_entry(ERC20MINTABLE_SIERRA_CLASS_HASH, ERC20MINTABLE_ABI_JSON),
    ]);

    let selectors = abi
        .values()
        .flat_map(|a| build_name_selector_map(a.clone()))
        .collect();

    ForkData { abi, selectors }
}
