use crate::forking::state::ForkStateReader;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spoof::TxInfoMock;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::{
    Event, SpyTarget,
};
use blockifier::execution::entry_point::CallEntryPoint;
use blockifier::state::state_api::State;
use blockifier::{
    execution::contract_class::ContractClass,
    state::{
        errors::StateError,
        state_api::{StateReader, StateResult},
    },
};
use cairo_felt::Felt252;
use runtime::starknet::context::BlockInfo;
use runtime::starknet::state::DictStateReader;

use starknet_api::core::EntryPointSelector;

use crate::constants::build_test_entry_point;
use starknet_api::transaction::ContractAddressSalt;
use starknet_api::{
    core::{ClassHash, CompiledClassHash, ContractAddress, Nonce},
    hash::StarkFelt,
    state::StorageKey,
};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::rc::Rc;

// Specifies which contracts to target
// with a cheatcode function
pub enum CheatTarget {
    All,
    One(ContractAddress),
    Multiple(Vec<ContractAddress>),
}

#[derive(Debug)]
pub struct ExtendedStateReader {
    pub dict_state_reader: DictStateReader,
    pub fork_state_reader: Option<ForkStateReader>,
}

pub trait BlockInfoReader {
    fn get_block_info(&mut self) -> StateResult<BlockInfo>;
}

impl BlockInfoReader for ExtendedStateReader {
    fn get_block_info(&mut self) -> StateResult<BlockInfo> {
        if let Some(ref mut fork_state_reader) = self.fork_state_reader {
            return fork_state_reader.get_block_info();
        }

        Ok(BlockInfo::default())
    }
}

pub struct BlockifierState<'a> {
    pub blockifier_state: &'a mut dyn State,
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

pub enum CheatStatus<T> {
    Cheated(T),
    Uncheated,
}

/// Tree structure representing trace of a call.
pub struct CallTrace {
    pub entry_point: CallEntryPoint,
    pub nested_calls: Vec<Rc<RefCell<CallTrace>>>,
}

pub struct NotEmptyCallStack(Vec<Rc<RefCell<CallTrace>>>);

impl NotEmptyCallStack {
    pub fn from(elem: Rc<RefCell<CallTrace>>) -> Self {
        NotEmptyCallStack(vec![elem])
    }

    pub fn push(&mut self, elem: Rc<RefCell<CallTrace>>) {
        self.0.push(elem);
    }

    pub fn top(&mut self) -> Rc<RefCell<CallTrace>> {
        let top_val = self.0.pop().unwrap();
        let borrowed_ref = top_val.clone();
        self.0.push(top_val);
        borrowed_ref
    }

    pub fn pop(&mut self) -> Rc<RefCell<CallTrace>> {
        assert!(self.0.len() > 1, "You cannot make NotEmptyCallStack empty");
        self.0.pop().unwrap()
    }

    #[must_use]
    pub fn borrow_full_trace(&self) -> Ref<'_, CallTrace> {
        self.0.first().unwrap().borrow()
    }
}

pub struct TraceData {
    pub current_call_stack: NotEmptyCallStack,
}

pub struct CheatnetState {
    pub rolled_contracts: HashMap<ContractAddress, CheatStatus<Felt252>>,
    pub global_roll: Option<Felt252>,
    pub pranked_contracts: HashMap<ContractAddress, CheatStatus<ContractAddress>>,
    pub global_prank: Option<ContractAddress>,
    pub warped_contracts: HashMap<ContractAddress, CheatStatus<Felt252>>,
    pub global_warp: Option<Felt252>,
    pub elected_contracts: HashMap<ContractAddress, CheatStatus<ContractAddress>>,
    pub global_elect: Option<ContractAddress>,
    pub mocked_functions: HashMap<ContractAddress, HashMap<EntryPointSelector, Vec<StarkFelt>>>,
    pub spoofed_contracts: HashMap<ContractAddress, CheatStatus<TxInfoMock>>,
    pub global_spoof: Option<TxInfoMock>,
    pub spies: Vec<SpyTarget>,
    pub detected_events: Vec<Event>,
    pub deploy_salt_base: u32,
    pub block_info: BlockInfo,
    // execution resources used by all contract calls
    pub used_resources: UsedResources,

    pub trace_data: TraceData,
}

impl Default for CheatnetState {
    fn default() -> Self {
        let test_call = Rc::new(RefCell::new(CallTrace {
            entry_point: build_test_entry_point(),
            nested_calls: vec![],
        }));
        Self {
            rolled_contracts: Default::default(),
            global_roll: None,
            pranked_contracts: Default::default(),
            global_prank: None,
            warped_contracts: Default::default(),
            global_warp: None,
            elected_contracts: Default::default(),
            global_elect: None,
            mocked_functions: Default::default(),
            spoofed_contracts: Default::default(),
            global_spoof: None,
            spies: vec![],
            detected_events: vec![],
            deploy_salt_base: 0,
            block_info: Default::default(),
            used_resources: Default::default(),
            trace_data: TraceData {
                current_call_stack: NotEmptyCallStack::from(test_call),
            },
        }
    }
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
    pub fn address_is_rolled(&self, contract_address: &ContractAddress) -> bool {
        self.get_cheated_block_number(contract_address).is_some()
    }

    #[must_use]
    pub fn address_is_warped(&self, contract_address: &ContractAddress) -> bool {
        self.get_cheated_block_timestamp(contract_address).is_some()
    }

    #[must_use]
    pub fn address_is_pranked(&self, contract_address: &ContractAddress) -> bool {
        self.get_cheated_caller_address(contract_address).is_some()
    }

    #[must_use]
    pub fn address_is_elected(&self, contract_address: &ContractAddress) -> bool {
        self.get_cheated_sequencer_address(contract_address)
            .is_some()
    }

    #[must_use]
    pub fn address_is_spoofed(&self, contract_address: &ContractAddress) -> bool {
        self.get_cheated_tx_info(contract_address).is_some()
    }

    #[must_use]
    pub fn get_cheated_block_number(&self, address: &ContractAddress) -> Option<Felt252> {
        get_cheat_for_contract(&self.global_roll, &self.rolled_contracts, address)
    }

    #[must_use]
    pub fn get_cheated_block_timestamp(&self, address: &ContractAddress) -> Option<Felt252> {
        get_cheat_for_contract(&self.global_warp, &self.warped_contracts, address)
    }

    #[must_use]
    pub fn get_cheated_sequencer_address(
        &self,
        address: &ContractAddress,
    ) -> Option<ContractAddress> {
        get_cheat_for_contract(&self.global_elect, &self.elected_contracts, address)
    }

    #[must_use]
    pub fn get_cheated_tx_info(&self, address: &ContractAddress) -> Option<TxInfoMock> {
        get_cheat_for_contract(&self.global_spoof, &self.spoofed_contracts, address)
    }

    #[must_use]
    pub fn get_cheated_caller_address(&self, address: &ContractAddress) -> Option<ContractAddress> {
        get_cheat_for_contract(&self.global_prank, &self.pranked_contracts, address)
    }
}

impl TraceData {
    pub fn enter_nested_call(&mut self, entry_point: CallEntryPoint) {
        let new_call = Rc::new(RefCell::new(CallTrace {
            entry_point,
            nested_calls: vec![],
        }));
        let current_call = self.current_call_stack.top();

        current_call
            .borrow_mut()
            .nested_calls
            .push(new_call.clone());

        self.current_call_stack.push(new_call);
    }

    pub fn exit_nested_call(&mut self) {
        self.current_call_stack.pop();
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

fn get_cheat_for_contract<T: Clone>(
    global_cheat: &Option<T>,
    contract_cheats: &HashMap<ContractAddress, CheatStatus<T>>,
    contract: &ContractAddress,
) -> Option<T> {
    if let Some(cheated_contract) = contract_cheats.get(contract) {
        match cheated_contract {
            CheatStatus::Cheated(contract_cheat) => Some(contract_cheat.clone()),
            CheatStatus::Uncheated => None,
        }
    } else {
        global_cheat.clone()
    }
}

pub fn start_cheat<T: Clone, S: BuildHasher>(
    global_cheat: &mut Option<T>,
    contract_cheats: &mut HashMap<ContractAddress, CheatStatus<T>, S>,
    target: CheatTarget,
    cheat_value: T,
) {
    match target {
        CheatTarget::All => {
            *global_cheat = Some(cheat_value);
            // Clear individual cheats so that `All`
            // contracts are affected by this cheat
            contract_cheats.clear();
        }
        CheatTarget::One(contract_address) => {
            (*contract_cheats).insert(contract_address, CheatStatus::Cheated(cheat_value));
        }
        CheatTarget::Multiple(contract_addresses) => {
            for contract_address in contract_addresses {
                (*contract_cheats)
                    .insert(contract_address, CheatStatus::Cheated(cheat_value.clone()));
            }
        }
    }
}

pub fn stop_cheat<T, S: BuildHasher>(
    global_cheat: &mut Option<T>,
    contract_cheats: &mut HashMap<ContractAddress, CheatStatus<T>, S>,
    target: CheatTarget,
) {
    match target {
        CheatTarget::All => {
            *global_cheat = None;
            contract_cheats.clear();
        }
        CheatTarget::One(contract_address) => {
            (*contract_cheats).insert(contract_address, CheatStatus::Uncheated);
        }
        CheatTarget::Multiple(contract_addresses) => {
            for contract_address in contract_addresses {
                (*contract_cheats).insert(contract_address, CheatStatus::Uncheated);
            }
        }
    };
}
