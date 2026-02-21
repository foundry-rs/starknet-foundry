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

mod tests {
    use super::ContractsRegistry;
    use starknet_types_core::felt::Felt;

    #[test]
    fn test_insert_and_get() {
        let mut registry = ContractsRegistry::new();
        let id = "contract1".to_string();
        let address = Felt::from(12345);

        assert!(
            registry
                .insert_new_id_to_address(id.clone(), address)
                .is_ok()
        );
        assert_eq!(registry.get_address_by_id(&id), Some(address));
    }

    #[test]
    fn test_duplicate_id() {
        let mut registry = ContractsRegistry::new();
        let id = "contract1".to_string();
        let address1 = Felt::from(12345);
        let address2 = Felt::from(67890);

        assert!(
            registry
                .insert_new_id_to_address(id.clone(), address1)
                .is_ok()
        );

        // Attempt to insert a duplicate id
        assert!(
            registry
                .insert_new_id_to_address(id.clone(), address2)
                .is_err()
        );

        // Ensure the original address is still retrievable
        assert_eq!(registry.get_address_by_id(&id), Some(address1));
    }
}
