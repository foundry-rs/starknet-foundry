use blockifier::{
    execution::contract_class::ContractClass,
    state::{
        cached_state::ContractStorageKey,
        errors::StateError,
        state_api::{StateReader, StateResult},
    },
};
use cairo_felt::Felt252;
use starknet_api::core::EntryPointSelector;
use starknet_api::{
    core::{ClassHash, CompiledClassHash, ContractAddress, Nonce},
    hash::StarkFelt,
    state::StorageKey,
};
use std::collections::HashMap;
use crate::cheatcodes;
use cheatcodes::spoof::TxInfoMock;

/// A simple implementation of `StateReader` using `HashMap`s as storage.
#[derive(Debug, Default)]
pub struct DictStateReader {
    pub storage_view: HashMap<ContractStorageKey, StarkFelt>,
    pub address_to_nonce: HashMap<ContractAddress, Nonce>,
    pub address_to_class_hash: HashMap<ContractAddress, ClassHash>,
    pub class_hash_to_class: HashMap<ClassHash, ContractClass>,
    pub class_hash_to_compiled_class_hash: HashMap<ClassHash, CompiledClassHash>,
}

impl StateReader for DictStateReader {
    fn get_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StorageKey,
    ) -> StateResult<StarkFelt> {
        let contract_storage_key = (contract_address, key);
        let value = self
            .storage_view
            .get(&contract_storage_key)
            .copied()
            .unwrap_or_default();
        Ok(value)
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        let nonce = self
            .address_to_nonce
            .get(&contract_address)
            .copied()
            .unwrap_or_default();
        Ok(nonce)
    }

    fn get_compiled_contract_class(
        &mut self,
        class_hash: &ClassHash,
    ) -> StateResult<ContractClass> {
        let contract_class = self.class_hash_to_class.get(class_hash).cloned();
        match contract_class {
            Some(contract_class) => Ok(contract_class),
            _ => Err(StateError::UndeclaredClassHash(*class_hash)),
        }
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        let class_hash = self
            .address_to_class_hash
            .get(&contract_address)
            .copied()
            .unwrap_or_default();
        Ok(class_hash)
    }

    fn get_compiled_class_hash(
        &mut self,
        class_hash: ClassHash,
    ) -> StateResult<starknet_api::core::CompiledClassHash> {
        let compiled_class_hash = self
            .class_hash_to_compiled_class_hash
            .get(&class_hash)
            .copied()
            .unwrap_or_default();
        Ok(compiled_class_hash)
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct CheatcodeState {
    pub rolled_contracts: HashMap<ContractAddress, Felt252>,
    pub pranked_contracts: HashMap<ContractAddress, ContractAddress>,
    pub warped_contracts: HashMap<ContractAddress, Felt252>,
    pub mocked_functions: HashMap<ContractAddress, HashMap<EntryPointSelector, Vec<StarkFelt>>>,
    pub spoofed_contracts: HashMap<ContractAddress, TxInfoMock>,
}

impl CheatcodeState {
    #[must_use]
    pub fn new() -> Self {
        CheatcodeState {
            rolled_contracts: HashMap::new(),
            pranked_contracts: HashMap::new(),
            warped_contracts: HashMap::new(),
            mocked_functions: HashMap::new(),
            spoofed_contracts: HashMap::new(),
        }
    }
}

impl Default for CheatcodeState {
    fn default() -> Self {
        Self::new()
    }
}
