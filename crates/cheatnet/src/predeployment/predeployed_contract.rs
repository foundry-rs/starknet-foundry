use crate::{ constants::contract_class};
use starknet_api::{
    contract_class::{ContractClass, SierraVersion},
    core::{ClassHash, ContractAddress},
    state::StorageKey,
};
use starknet_types_core::felt::Felt;

pub struct PredeployedContract {
    pub contract_address: ContractAddress,
    pub class_hash: ClassHash,
    pub contract_class: ContractClass,
    pub storage_kv_updates: Vec<(StorageKey, Felt)>,
}

impl PredeployedContract {
    #[must_use]
    pub fn new(
        contract_address: ContractAddress,
        class_hash: ClassHash,
        raw_casm: &str,
        storage_kv_updates: Vec<(StorageKey, Felt)>,
    ) -> Self {
        let contract_class = contract_class(raw_casm, SierraVersion::LATEST);
        Self {
            contract_address,
            class_hash,
            contract_class,
            storage_kv_updates,
        }
    }
}
