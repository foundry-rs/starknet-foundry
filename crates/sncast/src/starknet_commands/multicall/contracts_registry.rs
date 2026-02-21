use anyhow::Result;
use starknet_types_core::felt::Felt;
use std::collections::HashMap;

/// Registry for multicall execution, storing mappings from ids to contract addresses.
pub struct ContractsRegistry {
    id_to_address: HashMap<String, Felt>,
}

impl ContractsRegistry {
    pub fn new() -> Self {
        ContractsRegistry {
            id_to_address: HashMap::new(),
        }
    }

    /// Retrieves the contract address associated with the given id, if it exists.
    pub(crate) fn get_address_by_id(&self, id: &str) -> Option<Felt> {
        self.id_to_address.get(id).copied()
    }

    /// Inserts a mapping from the given id to the specified contract address.
    /// Returns an error if the id already exists.
    pub(crate) fn insert_new_id_to_address(&mut self, id: String, address: Felt) -> Result<()> {
        if self.id_to_address.contains_key(&id) {
            anyhow::bail!("Duplicate id found: {id}");
        }
        self.id_to_address.insert(id, address);
        Ok(())
    }
}
