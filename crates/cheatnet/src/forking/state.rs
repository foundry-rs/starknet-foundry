use crate::forking::worker::Worker;
use crate::state::DictStateReader;
use blockifier::execution::contract_class::ContractClass;
use blockifier::state::state_api::{StateReader, StateResult};
use starknet_api::core::{ClassHash, CompiledClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;

#[derive(Debug)]
pub struct ForkStateReader {
    pub dict_state_reader: DictStateReader,
    pub worker: Worker,
}

impl StateReader for ForkStateReader {
    fn get_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StorageKey,
    ) -> StateResult<StarkFelt> {
        let contract_storage_key = (contract_address, key);
        match self
            .dict_state_reader
            .storage_view
            .get(&contract_storage_key)
        {
            Some(value) => Ok(*value),
            None => Ok(self
                .worker
                .get_storage_at(contract_address, key)
                .unwrap_or_default()),
        }
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        match self
            .dict_state_reader
            .address_to_nonce
            .get(&contract_address)
        {
            Some(nonce) => Ok(*nonce),
            None => self.worker.get_nonce(contract_address),
        }
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        match self
            .dict_state_reader
            .address_to_class_hash
            .get(&contract_address)
        {
            Some(class_hash) => Ok(*class_hash),
            None => Ok(self
                .worker
                .get_class_hash_at(contract_address)
                .unwrap_or_default()),
        }
    }

    fn get_compiled_contract_class(
        &mut self,
        class_hash: &ClassHash,
    ) -> StateResult<ContractClass> {
        match self.dict_state_reader.class_hash_to_class.get(class_hash) {
            Some(compiled_class) => Ok(compiled_class.clone()),
            None => self.worker.get_compiled_contract_class(class_hash),
        }
    }

    fn get_compiled_class_hash(&mut self, class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        let compiled_class_hash = self
            .dict_state_reader
            .class_hash_to_compiled_class_hash
            .get(&class_hash)
            .copied()
            .unwrap_or_default();
        Ok(compiled_class_hash)
    }
}
