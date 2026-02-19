use anyhow::Result;
use sncast::{get_class_hash_by_address, get_contract_class};
use starknet_rust::{
    core::types::ContractClass,
    providers::{JsonRpcClient, jsonrpc::HttpTransport},
};
use starknet_types_core::felt::Felt;
use std::collections::HashMap;

pub(crate) struct ContractsCache {
    address_to_class_hash: HashMap<Felt, Felt>,
    class_hash_to_contract_class: HashMap<Felt, ContractClass>,
    provider: JsonRpcClient<HttpTransport>,
}

impl ContractsCache {
    fn new(provider: &JsonRpcClient<HttpTransport>) -> Self {
        ContractsCache {
            address_to_class_hash: HashMap::new(),
            class_hash_to_contract_class: HashMap::new(),
            provider: provider.clone(),
        }
    }

    /// Retrieves the class hash associated with the given contract address, if it exists.
    /// If not found in the cache, it queries the provider and updates the cache.
    pub(crate) async fn get_class_hash_by_address(&mut self, address: &Felt) -> Result<Felt> {
        if let Some(class_hash) = self.address_to_class_hash.get(address) {
            Ok(*class_hash)
        } else {
            let class_hash = get_class_hash_by_address(&self.provider, *address).await?;
            self.address_to_class_hash.insert(*address, class_hash);
            Ok(class_hash)
        }
    }

    /// Retrieves the contract class associated with the given class hash, if it exists.
    pub(crate) fn get_class_hash_by_address_local(&self, address: &Felt) -> Option<Felt> {
        self.address_to_class_hash.get(address).cloned()
    }

    /// Inserts a mapping from the given contract address to the specified class hash.
    /// Returns an error if the address already exists.
    pub(crate) fn try_insert_address_to_class_hash(
        &mut self,
        address: Felt,
        class_hash: Felt,
    ) -> Result<()> {
        if self.address_to_class_hash.contains_key(&address) {
            anyhow::bail!("Duplicate address found: {}", address);
        }
        self.address_to_class_hash.insert(address, class_hash);
        Ok(())
    }

    /// Retrieves the contract class associated with the given class hash, if it exists.
    /// If not found in the cache, it queries the provider and updates the cache.
    pub(crate) async fn get_contract_class_by_class_hash(
        &mut self,
        class_hash: &Felt,
    ) -> Result<ContractClass> {
        if let Some(contract_class) = self.class_hash_to_contract_class.get(class_hash) {
            Ok(contract_class.clone())
        } else {
            let contract_class = get_contract_class(*class_hash, &self.provider).await?;
            self.class_hash_to_contract_class
                .insert(*class_hash, contract_class.clone());
            Ok(contract_class)
        }
    }
}

/// Context for multicall execution, storing intermediate results and mappings between ids and contracts cache.
pub struct MulticallCtx {
    // calls: Vec<Call>,
    id_to_address: HashMap<String, Felt>,
    // address_to_class_hash: HashMap<Felt, Felt>,
    // class_hash_to_contract_class: HashMap<Felt, ContractClass>,
    pub(crate) cache: ContractsCache,
}

impl MulticallCtx {
    pub fn new(provider: &JsonRpcClient<HttpTransport>) -> Self {
        MulticallCtx {
            // calls: Vec::new(),
            id_to_address: HashMap::new(),
            // address_to_class_hash: HashMap::new(),
            // class_hash_to_contract_class: HashMap::new(),
            cache: ContractsCache::new(provider),
        }
    }

    /// Retrieves the contract address associated with the given id, if it exists.
    pub(crate) fn get_address_by_id(&self, id: &str) -> Option<Felt> {
        self.id_to_address.get(id).cloned()
    }

    /// Inserts a mapping from the given id to the specified contract address.
    /// Returns an error if the id already exists.
    pub(crate) fn try_insert_id_to_address(&mut self, id: String, address: Felt) -> Result<()> {
        if self.id_to_address.contains_key(&id) {
            anyhow::bail!("Duplicate id found: {}", id);
        }
        self.id_to_address.insert(id, address);
        Ok(())
    }
}
