use crate::forking::state::ForkStateReader;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::subtract_execution_resources;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spoof::TxInfoMock;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::{
    Event, SpyTarget,
};
use blockifier::execution::entry_point::{CallEntryPoint, ExecutionResources};
use blockifier::{
    execution::contract_class::ContractClass,
    state::state_api::{StateReader, StateResult},
};
use cairo_felt::Felt252;
use runtime::starknet::context::BlockInfo;
use runtime::starknet::state::DictStateReader;

use starknet_api::core::EntryPointSelector;

use crate::constants::build_test_entry_point;
use blockifier::state::errors::StateError::UndeclaredClassHash;
use starknet_api::transaction::ContractAddressSalt;
use starknet_api::{
    core::{ClassHash, CompiledClassHash, ContractAddress, Nonce},
    hash::StarkFelt,
    state::StorageKey,
};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::ops::Add;
use std::rc::Rc;

// Specifies which contracts to target
// with a cheatcode function
pub enum CheatTarget {
    All,
    One(ContractAddress),
    Multiple(Vec<ContractAddress>),
}

// Specifies the duration of the cheat
#[derive(Clone)]
pub enum CheatSpan {
    Indefinite,
    Number(usize),
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
                    .map_or(Ok(Default::default()), {
                        |reader| reader.get_storage_at(contract_address, key)
                    })
            })
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        self.dict_state_reader
            .get_nonce_at(contract_address)
            .or_else(|_| {
                self.fork_state_reader
                    .as_mut()
                    .map_or(Ok(Default::default()), {
                        |reader| reader.get_nonce_at(contract_address)
                    })
            })
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        self.dict_state_reader
            .get_class_hash_at(contract_address)
            .or_else(|_| {
                self.fork_state_reader
                    .as_mut()
                    .map_or(Ok(Default::default()), {
                        |reader| reader.get_class_hash_at(contract_address)
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
                self.fork_state_reader
                    .as_mut()
                    .map_or(Err(UndeclaredClassHash(*class_hash)), |reader| {
                        reader.get_compiled_contract_class(class_hash)
                    })
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
    Cheated(T, CheatSpan),
    Uncheated,
}

impl<T> CheatStatus<T> {
    pub fn decrement_cheat_span(&mut self) {
        if let CheatStatus::Cheated(_, CheatSpan::Number(n)) = self {
            *n -= 1;
            if *n == 0 {
                *self = CheatStatus::Uncheated;
            }
        }
    }
}

/// Tree structure representing trace of a call.
#[derive(Clone)]
pub struct CallTrace {
    pub entry_point: CallEntryPoint,
    // These also include resources used by internal calls
    pub used_execution_resources: ExecutionResources,
    pub nested_calls: Vec<Rc<RefCell<CallTrace>>>,
}

#[derive(Clone)]
struct CallStackElement {
    // when we exit the call we use it to calculate resources used by the call
    resources_used_before_call: ExecutionResources,
    call_trace: Rc<RefCell<CallTrace>>,
}

pub struct NotEmptyCallStack(Vec<CallStackElement>);

impl NotEmptyCallStack {
    pub fn from(elem: Rc<RefCell<CallTrace>>) -> Self {
        NotEmptyCallStack(vec![CallStackElement {
            resources_used_before_call: ExecutionResources::default(),
            call_trace: elem,
        }])
    }

    pub fn push(
        &mut self,
        elem: Rc<RefCell<CallTrace>>,
        resources_used_before_call: ExecutionResources,
    ) {
        self.0.push(CallStackElement {
            resources_used_before_call,
            call_trace: elem,
        });
    }

    pub fn top(&mut self) -> Rc<RefCell<CallTrace>> {
        let top_val = self.0.pop().unwrap();
        let borrowed_ref = top_val.call_trace.clone();
        self.0.push(top_val);
        borrowed_ref
    }

    fn pop(&mut self) -> CallStackElement {
        assert!(self.0.len() > 1, "You cannot make NotEmptyCallStack empty");
        self.0.pop().unwrap()
    }

    #[must_use]
    pub fn borrow_full_trace(&self) -> Ref<'_, CallTrace> {
        self.0.first().unwrap().call_trace.borrow()
    }
}

pub struct TraceData {
    pub current_call_stack: NotEmptyCallStack,
}

pub struct CheatnetState {
    pub rolled_contracts: HashMap<ContractAddress, CheatStatus<Felt252>>,
    pub global_roll: Option<(Felt252, CheatSpan)>,
    pub pranked_contracts: HashMap<ContractAddress, CheatStatus<ContractAddress>>,
    pub global_prank: Option<(ContractAddress, CheatSpan)>,
    pub warped_contracts: HashMap<ContractAddress, CheatStatus<Felt252>>,
    pub global_warp: Option<(Felt252, CheatSpan)>,
    pub elected_contracts: HashMap<ContractAddress, CheatStatus<ContractAddress>>,
    pub global_elect: Option<(ContractAddress, CheatSpan)>,
    pub mocked_functions: HashMap<ContractAddress, HashMap<EntryPointSelector, Vec<StarkFelt>>>,
    pub spoofed_contracts: HashMap<ContractAddress, CheatStatus<TxInfoMock>>,
    pub global_spoof: Option<(TxInfoMock, CheatSpan)>,
    pub spies: Vec<SpyTarget>,
    pub detected_events: Vec<Event>,
    pub deploy_salt_base: u32,
    pub block_info: BlockInfo,
    pub trace_data: TraceData,
}

impl Default for CheatnetState {
    fn default() -> Self {
        let test_call = Rc::new(RefCell::new(CallTrace {
            entry_point: build_test_entry_point(),
            used_execution_resources: Default::default(),
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

    pub fn get_and_update_cheated_block_number(
        &mut self,
        address: &ContractAddress,
    ) -> Option<Felt252> {
        get_and_update_cheat_for_contract(&self.global_roll, &mut self.rolled_contracts, address)
    }

    #[must_use]
    pub fn get_cheated_block_timestamp(&self, address: &ContractAddress) -> Option<Felt252> {
        get_cheat_for_contract(&self.global_warp, &self.warped_contracts, address)
    }

    pub fn get_and_update_cheated_block_timestamp(
        &mut self,
        address: &ContractAddress,
    ) -> Option<Felt252> {
        get_and_update_cheat_for_contract(&self.global_warp, &mut self.warped_contracts, address)
    }

    #[must_use]
    pub fn get_cheated_sequencer_address(
        &self,
        address: &ContractAddress,
    ) -> Option<ContractAddress> {
        get_cheat_for_contract(&self.global_elect, &self.elected_contracts, address)
    }

    pub fn get_and_update_cheated_sequencer_address(
        &mut self,
        address: &ContractAddress,
    ) -> Option<ContractAddress> {
        get_and_update_cheat_for_contract(&self.global_elect, &mut self.elected_contracts, address)
    }

    #[must_use]
    pub fn get_cheated_tx_info(&self, address: &ContractAddress) -> Option<TxInfoMock> {
        get_cheat_for_contract(&self.global_spoof, &self.spoofed_contracts, address)
    }

    pub fn get_and_update_cheated_tx_info(
        &mut self,
        address: &ContractAddress,
    ) -> Option<TxInfoMock> {
        get_and_update_cheat_for_contract(&self.global_spoof, &mut self.spoofed_contracts, address)
    }

    #[must_use]
    pub fn get_cheated_caller_address(&self, address: &ContractAddress) -> Option<ContractAddress> {
        get_cheat_for_contract(&self.global_prank, &self.pranked_contracts, address)
    }

    pub fn get_and_update_cheated_caller_address(
        &mut self,
        address: &ContractAddress,
    ) -> Option<ContractAddress> {
        get_and_update_cheat_for_contract(&self.global_prank, &mut self.pranked_contracts, address)
    }
}

impl TraceData {
    pub fn enter_nested_call(
        &mut self,
        entry_point: CallEntryPoint,
        resources_used_before_call: ExecutionResources,
    ) {
        let new_call = Rc::new(RefCell::new(CallTrace {
            entry_point,
            used_execution_resources: Default::default(),
            nested_calls: vec![],
        }));
        let current_call = self.current_call_stack.top();

        current_call
            .borrow_mut()
            .nested_calls
            .push(new_call.clone());

        self.current_call_stack
            .push(new_call, resources_used_before_call);
    }

    pub fn exit_nested_call(&mut self, resources_used_after_call: &ExecutionResources) {
        let CallStackElement {
            resources_used_before_call,
            call_trace: last_call,
        } = self.current_call_stack.pop();

        last_call.borrow_mut().used_execution_resources =
            subtract_execution_resources(resources_used_after_call, &resources_used_before_call);
    }
}

fn get_and_update_cheat_for_contract<T: Clone>(
    global_cheat: &Option<(T, CheatSpan)>,
    contract_cheats: &mut HashMap<ContractAddress, CheatStatus<T>>,
    contract: &ContractAddress,
) -> Option<T> {
    let cheated_value = get_cheat_for_contract(global_cheat, contract_cheats, contract);
    update_cheat_for_contract(global_cheat, contract_cheats, contract);
    cheated_value
}

fn get_cheat_for_contract<T: Clone>(
    global_cheat: &Option<(T, CheatSpan)>,
    contract_cheats: &HashMap<ContractAddress, CheatStatus<T>>,
    contract: &ContractAddress,
) -> Option<T> {
    if let Some(cheat_status) = contract_cheats.get(contract) {
        match cheat_status {
            CheatStatus::Cheated(contract_cheat, _) => Some(contract_cheat.clone()),
            CheatStatus::Uncheated => None,
        }
    } else {
        global_cheat.as_ref().map(|(cheat, _)| cheat.clone())
    }
}

fn update_cheat_for_contract<T: Clone>(
    global_cheat: &Option<(T, CheatSpan)>,
    contract_cheats: &mut HashMap<ContractAddress, CheatStatus<T>>,
    contract: &ContractAddress,
) {
    if let Some(cheat_status) = contract_cheats.get_mut(contract) {
        cheat_status.decrement_cheat_span();
    } else if let Some((cheat, span)) = global_cheat {
        let mut cheat_status = CheatStatus::Cheated(cheat.clone(), span.clone());
        cheat_status.decrement_cheat_span();
        contract_cheats.insert(*contract, cheat_status);
    }
}

pub fn start_cheat<T: Clone, S: BuildHasher>(
    global_cheat: &mut Option<(T, CheatSpan)>,
    contract_cheats: &mut HashMap<ContractAddress, CheatStatus<T>, S>,
    target: CheatTarget,
    cheat_value: T,
    span: CheatSpan,
) {
    match target {
        CheatTarget::All => {
            *global_cheat = Some((cheat_value, span));
            // Clear individual cheats so that `All`
            // contracts are affected by this cheat
            contract_cheats.clear();
        }
        CheatTarget::One(contract_address) => {
            (*contract_cheats).insert(contract_address, CheatStatus::Cheated(cheat_value, span));
        }
        CheatTarget::Multiple(contract_addresses) => {
            for contract_address in contract_addresses {
                (*contract_cheats).insert(
                    contract_address,
                    CheatStatus::Cheated(cheat_value.clone(), span.clone()),
                );
            }
        }
    };
}

pub fn stop_cheat<T, S: BuildHasher>(
    global_cheat: &mut Option<(T, CheatSpan)>,
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
