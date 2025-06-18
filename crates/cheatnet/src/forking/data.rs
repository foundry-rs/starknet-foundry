use crate::runtime_extensions::forge_runtime_extension::contracts_data::{
    FunctionName, build_name_selector_map,
};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use starknet::core::types::ContractClass;
use starknet::core::types::contract::AbiEntry;
use starknet_api::core::{ClassHash, EntryPointSelector};
use std::collections::HashMap;

#[derive(Default)]
pub struct ForksData {
    pub abi: HashMap<ClassHash, Vec<AbiEntry>>,
    pub selectors: HashMap<EntryPointSelector, FunctionName>,
}

impl ForksData {
    /// Creates a new instance of [`ForksData`] from a map of [`ClassHash`] to their [`ContractClass`].
    #[must_use]
    pub fn new(fork_compiled_contract_class_map: &HashMap<ClassHash, ContractClass>) -> Self {
        let abi = fork_compiled_contract_class_map
            .par_iter()
            .filter_map(|(class_hash, contract_class)| {
                let ContractClass::Sierra(sierra_class) = contract_class else {
                    return None;
                };
                let abi = serde_json::from_str::<Vec<AbiEntry>>(&sierra_class.abi)
                    .expect("this should be valid `ABI`");
                Some((*class_hash, abi))
            })
            .collect::<HashMap<_, _>>();

        let selectors = abi
            .iter()
            .flat_map(|(_, abi)| build_name_selector_map(abi.clone()))
            .collect();

        Self { abi, selectors }
    }
}
