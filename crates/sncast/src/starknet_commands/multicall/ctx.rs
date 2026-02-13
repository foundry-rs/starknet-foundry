use anyhow::Result;
use starknet_rust::core::types::Call;
use starknet_types_core::felt::Felt;
use std::collections::HashMap;

/// Context for multicall execution, storing calls and mappings between ids, addresses and class hashes.
#[derive(Default)]
pub struct MulticallCtx {
    calls: Vec<Call>,
    id_to_address: HashMap<String, Felt>,
    address_to_class_hash: HashMap<Felt, Felt>,
}

impl MulticallCtx {
    /// Retrieves the contract address associated with the given id, if it exists.
    pub fn get_address_by_id(&self, id: &str) -> Option<Felt> {
        self.id_to_address.get(id).cloned()
    }

    /// Inserts a mapping from the given id to the specified contract address. Returns an error if the id already exists.
    pub fn insert_id_to_address(&mut self, id: String, address: Felt) -> Result<()> {
        if self.id_to_address.contains_key(&id) {
            anyhow::bail!("Duplicate id found: {}", id);
        }
        self.id_to_address.insert(id, address);
        Ok(())
    }

    /// Retrieves the class hash associated with the given contract address, if it exists.
    pub fn get_class_hash_by_address(&self, address: &Felt) -> Option<Felt> {
        self.address_to_class_hash.get(address).cloned()
    }

    /// Inserts a mapping from the given contract address to the specified class hash. Returns an error if the address already exists.
    pub fn insert_address_class_hash_mapping(
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

    /// Returns a reference to the list of calls stored in the context.
    pub fn calls(&self) -> &[Call] {
        &self.calls
    }

    /// Adds a new call to the context's list of calls.
    pub fn add_call(&mut self, call: Call) {
        self.calls.push(call);
    }
}
