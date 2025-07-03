use crate::trace::types::{ContractName, Selector};
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::{
    ContractsData as CheatnetContractsData, build_name_selector_map,
};
use cheatnet::sync_client::SyncClient;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use starknet::core::types::ContractClass;
use starknet::core::types::contract::{AbiEntry, SierraClass};
use starknet::providers::ProviderError;
use starknet_api::core::{ClassHash, EntryPointSelector};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(thiserror::Error, Debug, Clone)]
pub enum NetworkLookupError {
    #[error(transparent)]
    ProviderError(#[from] Arc<ProviderError>),
    #[error(
        "Legacy contract class at class hash {0}. Cairo 0 contracts are not supported in this context."
    )]
    LegacyContractClass(ClassHash),
}

/// Data structure containing information about contracts,
/// including their ABI, names, and selectors that will be used to create a [`Trace`](crate::Trace).
/// Contains methods to fetch and cache this information from the network if not already present.
pub struct ContractsDataStore {
    abi: HashMap<ClassHash, Vec<AbiEntry>>,
    contract_names: HashMap<ClassHash, ContractName>,
    selectors: HashMap<EntryPointSelector, Selector>,
    client: Option<SyncClient>,
}

impl ContractsDataStore {
    /// Creates a new instance of [`ContractsDataStore`] from a similar structure from `cheatnet`: [`CheatnetContractsData`] and [`Option<SyncClient>`].
    #[must_use]
    pub fn new(
        cheatnet_contracts_data: &CheatnetContractsData,
        client: Option<SyncClient>,
    ) -> Self {
        let contract_names = cheatnet_contracts_data
            .class_hashes
            .iter()
            .map(|(name, class_hash)| (*class_hash, ContractName(name.clone())))
            .collect();

        let selectors = cheatnet_contracts_data
            .selectors
            .iter()
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
            .collect();

        Self {
            abi,
            contract_names,
            selectors,
            client,
        }
    }

    /// Gets the [`ContractName`] for a given contract [`ClassHash`].
    #[must_use]
    pub fn get_contract_name(&self, class_hash: &ClassHash) -> Option<&ContractName> {
        self.contract_names.get(class_hash)
    }

    /// Gets the `abi` for a given contract [`ClassHash`] from [`ContractsDataStore`]
    /// and if it is not present fetches it from network and caches it.
    pub fn get_or_fetch_abi(
        &mut self,
        class_hash: &ClassHash,
    ) -> Result<&[AbiEntry], NetworkLookupError> {
        if !self.abi.contains_key(class_hash) {
            self.network_lookup(class_hash)?;
        }
        Ok(self
            .abi
            .get(class_hash)
            .map(Vec::as_slice)
            .expect("ABI should be present after network lookup"))
    }

    /// Gets the [`Selector`] in human-readable form for a given [`EntryPointSelector`] from [`ContractsDataStore`]
    /// and if it is not present fetches it from network and caches it.
    pub fn get_or_fetch_selector(
        &mut self,
        entry_point_selector: &EntryPointSelector,
        class_hash: &ClassHash,
    ) -> Result<&Selector, NetworkLookupError> {
        if !self.selectors.contains_key(entry_point_selector) {
            self.network_lookup(class_hash)?;
        }
        Ok(self
            .selectors
            .get(entry_point_selector)
            .expect("Selector should be present after network lookup"))
    }

    /// Adds a new contract class to the [`ContractsDataStore`] by looking it up on the network.
    fn network_lookup(&mut self, class_hash: &ClassHash) -> Result<(), NetworkLookupError> {
        let client = self
            .client
            .as_ref()
            .expect("if this function is called, then `client` must be set");

        let class = client.get_class(class_hash.0).map_err(Arc::new)?;
        let ContractClass::Sierra(sierra_class) = class else {
            return Err(NetworkLookupError::LegacyContractClass(*class_hash));
        };

        let abi = serde_json::from_str::<Vec<AbiEntry>>(&sierra_class.abi)
            .expect("this should be valid ABI");

        let selectors = build_name_selector_map(abi.clone())
            .into_iter()
            .map(|(selector, name)| (selector, Selector(name)));

        self.abi.insert(*class_hash, abi);
        self.selectors.extend(selectors);
        Ok(())
    }
}
