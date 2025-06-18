use crate::trace::types::{ContractName, Selector};
use cheatnet::forking::data::ForksData;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData as CheatnetContractsData;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use starknet::core::types::contract::{AbiEntry, SierraClass};
use starknet_api::core::{ClassHash, EntryPointSelector};
use std::collections::HashMap;

/// Data structure containing information about contracts,
/// including their ABI, names, and selectors that will be used to create a [`Trace`](crate::Trace).
pub struct ContractsDataStore {
    abi: HashMap<ClassHash, Vec<AbiEntry>>,
    contract_names: HashMap<ClassHash, ContractName>,
    selectors: HashMap<EntryPointSelector, Selector>,
}

impl ContractsDataStore {
    /// Creates a new instance of [`ContractsDataStore`] from a similar structure from `cheatnet`: [`CheatnetContractsData`]
    /// and [`ForksData`].
    #[must_use]
    pub fn new(cheatnet_contracts_data: &CheatnetContractsData, forks_data: ForksData) -> Self {
        let contract_names = cheatnet_contracts_data
            .class_hashes
            .iter()
            .map(|(name, class_hash)| (*class_hash, ContractName(name.clone())))
            .collect();

        let selectors = cheatnet_contracts_data
            .selectors
            .iter()
            .chain(&forks_data.selectors)
            .map(|(selector, function_name)| (*selector, Selector(function_name.clone())))
            .collect();

        let abi = cheatnet_contracts_data
            .contracts
            .par_iter()
            .map(|(_, contract_data)| {
                let sierra = serde_json::from_str::<SierraClass>(&contract_data.artifacts.sierra)
                    .expect("this should be valid `SierraClass`");
                (contract_data.class_hash, sierra.abi)
            })
            .chain(forks_data.abi)
            .collect();

        Self {
            abi,
            contract_names,
            selectors,
        }
    }

    /// Gets the [`ContractName`] for a given contract [`ClassHash`].
    #[must_use]
    pub fn get_contract_name(&self, class_hash: &ClassHash) -> Option<&ContractName> {
        self.contract_names.get(class_hash)
    }

    /// Gets the `abi` for a given contract [`ClassHash`].
    #[must_use]
    pub fn get_abi(&self, class_hash: &ClassHash) -> Option<&[AbiEntry]> {
        self.abi.get(class_hash).map(Vec::as_slice)
    }

    /// Gets the [`Selector`] in human-readable form for a given [`EntryPointSelector`].
    #[must_use]
    pub fn get_selector(&self, entry_point_selector: &EntryPointSelector) -> Option<&Selector> {
        self.selectors.get(entry_point_selector)
    }
}
