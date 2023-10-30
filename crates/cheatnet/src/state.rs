use crate::cheatcodes;
use crate::cheatcodes::spy_events::{Event, SpyTarget};
use crate::constants::TEST_SEQUENCER_ADDRESS;
use crate::forking::state::ForkStateReader;
use blockifier::state::state_api::State;
use blockifier::{
    execution::contract_class::ContractClass,
    state::{
        cached_state::ContractStorageKey,
        errors::StateError,
        state_api::{StateReader, StateResult},
    },
};
use cairo_felt::Felt252;
use cheatcodes::spoof::TxInfoMock;
use serde::{Deserialize, Serialize};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::core::EntryPointSelector;
use starknet_api::core::PatriciaKey;
use starknet_api::hash::StarkHash;
use starknet_api::transaction::ContractAddressSalt;
use starknet_api::{
    core::{ClassHash, CompiledClassHash, ContractAddress, Nonce},
    hash::StarkFelt,
    patricia_key,
    state::StorageKey,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct ExtendedStateReader {
    pub dict_state_reader: DictStateReader,
    pub fork_state_reader: Option<ForkStateReader>,
}

pub trait BlockInfoReader {
    fn get_block_info(&mut self) -> StateResult<CheatnetBlockInfo>;
}

impl BlockInfoReader for ExtendedStateReader {
    fn get_block_info(&mut self) -> StateResult<CheatnetBlockInfo> {
        if let Some(ref mut fork_state_reader) = self.fork_state_reader {
            return fork_state_reader.get_block_info();
        }

        Ok(CheatnetBlockInfo::default())
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct BlockifierState<'a> {
    pub blockifier_state: &'a mut dyn State,
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct CheatnetBlockInfo {
    pub block_number: BlockNumber,
    pub timestamp: BlockTimestamp,
    pub sequencer_address: ContractAddress,
}

impl Default for CheatnetBlockInfo {
    fn default() -> Self {
        Self {
            block_number: BlockNumber(2000),
            timestamp: BlockTimestamp::default(),
            sequencer_address: ContractAddress(patricia_key!(TEST_SEQUENCER_ADDRESS)),
        }
    }
}

impl<'a> BlockifierState<'a> {
    pub fn from(state: &'a mut dyn State) -> Self {
        BlockifierState {
            blockifier_state: state,
        }
    }
}

impl StateReader for ExtendedStateReader {
    fn get_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StorageKey,
    ) -> StateResult<StarkFelt> {
        self.dict_state_reader
            .get_storage_at(contract_address, key)
            .or_else(|_| {
                self.fork_state_reader
                    .as_mut()
                    .map_or(Ok(StarkFelt::default()), |reader| {
                        match_node_response(reader.get_storage_at(contract_address, key))
                    })
            })
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        self.dict_state_reader
            .get_nonce_at(contract_address)
            .or_else(|_| {
                self.fork_state_reader
                    .as_mut()
                    .map_or(Ok(Nonce::default()), |reader| {
                        match_node_response(reader.get_nonce_at(contract_address))
                    })
            })
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        self.dict_state_reader
            .get_class_hash_at(contract_address)
            .or_else(|_| {
                self.fork_state_reader
                    .as_mut()
                    .map_or(Ok(ClassHash::default()), |reader| {
                        match_node_response(reader.get_class_hash_at(contract_address))
                    })
            })
    }

    fn get_compiled_contract_class(
        &mut self,
        class_hash: &ClassHash,
    ) -> StateResult<ContractClass> {
        self.dict_state_reader
            .get_compiled_contract_class(class_hash)
            .or_else(|_| {
                self.fork_state_reader.as_mut().map_or(
                    Err(StateError::UndeclaredClassHash(*class_hash)),
                    |reader| reader.get_compiled_contract_class(class_hash),
                )
            })
    }

    fn get_compiled_class_hash(&mut self, class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        Ok(self
            .dict_state_reader
            .get_compiled_class_hash(class_hash)
            .unwrap_or_default())
    }
}

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
        self.storage_view
            .get(&contract_storage_key)
            .copied()
            .ok_or(StateError::StateReadError(format!(
                "Unable to get storage at address: {contract_address:?} and key: {key:?} form DictStateReader"
            )))
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        self.address_to_nonce
            .get(&contract_address)
            .copied()
            .ok_or(StateError::StateReadError(format!(
                "Unable to get nonce at {contract_address:?} from DictStateReader"
            )))
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        self.address_to_class_hash
            .get(&contract_address)
            .copied()
            .ok_or(StateError::UnavailableContractAddress(contract_address))
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

    fn get_compiled_class_hash(&mut self, class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        let compiled_class_hash = self
            .class_hash_to_compiled_class_hash
            .get(&class_hash)
            .copied()
            .unwrap_or_default();
        Ok(compiled_class_hash)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Default)]
pub struct CheatnetState {
    pub rolled_contracts: HashMap<ContractAddress, Felt252>,
    pub pranked_contracts: HashMap<ContractAddress, ContractAddress>,
    pub warped_contracts: HashMap<ContractAddress, Felt252>,
    pub mocked_functions: HashMap<ContractAddress, HashMap<EntryPointSelector, Vec<StarkFelt>>>,
    pub spoofed_contracts: HashMap<ContractAddress, TxInfoMock>,
    pub spies: Vec<SpyTarget>,
    pub detected_events: Vec<Event>,
    pub deploy_salt_base: u32,
    pub block_info: CheatnetBlockInfo,
}

impl CheatnetState {
    pub fn increment_deploy_salt_base(&mut self) {
        self.deploy_salt_base += 1;
    }

    #[must_use]
    pub fn get_salt(&self) -> ContractAddressSalt {
        ContractAddressSalt(StarkFelt::from(self.deploy_salt_base))
    }

    #[must_use]
    pub fn address_is_pranked(&self, contract_address: &ContractAddress) -> bool {
        self.pranked_contracts.contains_key(contract_address)
    }

    #[must_use]
    pub fn address_is_warped(&self, contract_address: &ContractAddress) -> bool {
        self.warped_contracts.contains_key(contract_address)
    }

    #[must_use]
    pub fn address_is_rolled(&self, contract_address: &ContractAddress) -> bool {
        self.rolled_contracts.contains_key(contract_address)
    }

    #[must_use]
    pub fn address_is_spoofed(&self, contract_address: &ContractAddress) -> bool {
        self.spoofed_contracts.contains_key(contract_address)
    }

    #[must_use]
    pub fn address_is_cheated(&self, contract_address: &ContractAddress) -> bool {
        self.address_is_rolled(contract_address)
            || self.address_is_pranked(contract_address)
            || self.address_is_warped(contract_address)
            || self.address_is_spoofed(contract_address)
    }
}

fn match_node_response<T: Default>(result: StateResult<T>) -> StateResult<T> {
    match result {
        Ok(class_hash) => Ok(class_hash),
        Err(StateError::StateReadError(msg)) if msg.contains("node") => {
            Err(StateError::StateReadError(msg))
        }
        _ => Ok(Default::default()),
    }
}
