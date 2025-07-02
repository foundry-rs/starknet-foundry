use crate::runtime_extensions::forge_runtime_extension::contracts_data::build_name_selector_map;
use starknet::core::types::ContractClass;
use starknet::core::types::contract::AbiEntry;
use starknet_api::core::{ClassHash, EntryPointSelector};
use std::collections::HashMap;

#[derive(Default)]
pub struct ForkData {
    pub abi: HashMap<ClassHash, Vec<AbiEntry>>,
    pub selectors: HashMap<EntryPointSelector, String>,
}

impl ForkData {
    /// Creates a new instance of [`ForkData`] from a given `fork_compiled_contract_class`.
    #[must_use]
    pub fn new(fork_compiled_contract_class: &HashMap<ClassHash, ContractClass>) -> Self {
        let abi: HashMap<ClassHash, Vec<AbiEntry>> = fork_compiled_contract_class
            .iter()
            .filter_map(|(class_hash, contract_class)| {
                let ContractClass::Sierra(sierra_class) = contract_class else {
                    return None;
                };

                let abi = serde_json::from_str::<Vec<AbiEntry>>(&sierra_class.abi)
                    .expect("this should be valid ABI");

                Some((*class_hash, abi))
            })
            .collect();

        let selectors = abi
            .iter()
            .flat_map(|(_, abi)| build_name_selector_map(abi.clone()))
            .collect();

        Self { abi, selectors }
    }
}
