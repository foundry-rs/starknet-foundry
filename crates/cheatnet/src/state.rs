use crate::forking::state::ForkStateReader;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallResult;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spoof::TxInfoMock;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::{
    Event, SpyTarget,
};
use blockifier::execution::entry_point::CallEntryPoint;
use blockifier::{
    execution::contract_class::ContractClass,
    state::state_api::{StateReader, StateResult},
};
use cairo_felt::Felt252;
use runtime::starknet::state::DictStateReader;

use starknet_api::core::EntryPointSelector;

use crate::constants::{build_test_entry_point, TEST_CONTRACT_CLASS_HASH};
use blockifier::blockifier::block::BlockInfo;
use blockifier::execution::call_info::OrderedL2ToL1Message;
use blockifier::execution::syscalls::hint_processor::SyscallCounter;
use blockifier::state::errors::StateError::UndeclaredClassHash;
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use cairo_vm::vm::trace::trace_entry::TraceEntry;
use runtime::starknet::context::SerializableBlockInfo;
use starknet_api::transaction::ContractAddressSalt;
use starknet_api::{
    class_hash,
    core::{ClassHash, CompiledClassHash, ContractAddress, Nonce},
    hash::{StarkFelt, StarkHash},
    state::StorageKey,
};
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::rc::Rc;
use trace_data::L1Resources;

// Specifies which contracts to target
// with a cheatcode function
pub enum CheatTarget {
    All,
    One(ContractAddress),
    Multiple(Vec<ContractAddress>),
}

// Specifies the duration of the cheat
#[derive(Clone, Debug)]
pub enum CheatSpan {
    Indefinite,
    TargetCalls(usize),
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

        Ok(SerializableBlockInfo::default().into())
    }
}

impl StateReader for ExtendedStateReader {
    fn get_storage_at(
        &self,
        contract_address: ContractAddress,
        key: StorageKey,
    ) -> StateResult<StarkFelt> {
        self.dict_state_reader
            .get_storage_at(contract_address, key)
            .or_else(|_| {
                self.fork_state_reader
                    .as_ref()
                    .map_or(Ok(Default::default()), {
                        |reader| reader.get_storage_at(contract_address, key)
                    })
            })
    }

    fn get_nonce_at(&self, contract_address: ContractAddress) -> StateResult<Nonce> {
        self.dict_state_reader
            .get_nonce_at(contract_address)
            .or_else(|_| {
                self.fork_state_reader
                    .as_ref()
                    .map_or(Ok(Default::default()), {
                        |reader| reader.get_nonce_at(contract_address)
                    })
            })
    }

    fn get_class_hash_at(&self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        self.dict_state_reader
            .get_class_hash_at(contract_address)
            .or_else(|_| {
                self.fork_state_reader
                    .as_ref()
                    .map_or(Ok(Default::default()), {
                        |reader| reader.get_class_hash_at(contract_address)
                    })
            })
    }

    fn get_compiled_contract_class(&self, class_hash: ClassHash) -> StateResult<ContractClass> {
        self.dict_state_reader
            .get_compiled_contract_class(class_hash)
            .or_else(|_| {
                self.fork_state_reader
                    .as_ref()
                    .map_or(Err(UndeclaredClassHash(class_hash)), |reader| {
                        reader.get_compiled_contract_class(class_hash)
                    })
            })
    }

    fn get_compiled_class_hash(&self, class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        Ok(self
            .dict_state_reader
            .get_compiled_class_hash(class_hash)
            .unwrap_or_default())
    }
}

#[derive(Debug, Clone)]
pub enum CheatStatus<T> {
    Cheated(T, CheatSpan),
    Uncheated,
}

impl<T> CheatStatus<T> {
    pub fn decrement_cheat_span(&mut self) {
        if let CheatStatus::Cheated(_, CheatSpan::TargetCalls(n)) = self {
            *n -= 1;
            if *n == 0 {
                *self = CheatStatus::Uncheated;
            }
        }
    }
}

/// Tree structure representing trace of a call.
pub struct CallTrace {
    pub entry_point: CallEntryPoint,
    // These also include resources used by internal calls
    pub used_execution_resources: ExecutionResources,
    pub used_l1_resources: L1Resources,
    pub used_syscalls: SyscallCounter,
    pub nested_calls: Vec<CallTraceNode>,
    pub result: CallResult,
    pub vm_trace: Option<Vec<TraceEntry>>,
}

impl CallTrace {
    fn default_successful_call() -> Self {
        Self {
            entry_point: Default::default(),
            used_execution_resources: Default::default(),
            used_l1_resources: Default::default(),
            used_syscalls: Default::default(),
            nested_calls: vec![],
            result: CallResult::Success { ret_data: vec![] },
            vm_trace: None,
        }
    }
}

/// Enum representing node of a trace of a call.
#[derive(Clone)]
pub enum CallTraceNode {
    EntryPointCall(Rc<RefCell<CallTrace>>),
    DeployWithoutConstructor,
}

impl CallTraceNode {
    #[must_use]
    pub fn extract_entry_point_call(&self) -> Option<&Rc<RefCell<CallTrace>>> {
        if let CallTraceNode::EntryPointCall(trace) = self {
            Some(trace)
        } else {
            None
        }
    }
}

#[derive(Clone)]
struct CallStackElement {
    // when we exit the call we use it to calculate resources used by the call
    resources_used_before_call: ExecutionResources,
    call_trace: Rc<RefCell<CallTrace>>,
    cheated_data: CheatedData,
}

pub struct NotEmptyCallStack(Vec<CallStackElement>);

impl NotEmptyCallStack {
    pub fn from(elem: Rc<RefCell<CallTrace>>) -> Self {
        NotEmptyCallStack(vec![CallStackElement {
            resources_used_before_call: ExecutionResources::default(),
            call_trace: elem,
            cheated_data: Default::default(),
        }])
    }

    pub fn push(
        &mut self,
        elem: Rc<RefCell<CallTrace>>,
        resources_used_before_call: ExecutionResources,
        cheated_data: CheatedData,
    ) {
        self.0.push(CallStackElement {
            resources_used_before_call,
            call_trace: elem,
            cheated_data,
        });
    }

    pub fn top(&mut self) -> Rc<RefCell<CallTrace>> {
        let top_val = self.0.last().unwrap();
        top_val.call_trace.clone()
    }

    pub fn top_cheated_data(&mut self) -> CheatedData {
        let top_val = self.0.last().unwrap();
        top_val.cheated_data.clone()
    }

    fn pop(&mut self) -> CallStackElement {
        assert!(self.0.len() > 1, "You cannot make NotEmptyCallStack empty");
        self.0.pop().unwrap()
    }

    #[must_use]
    pub fn size(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn borrow_full_trace(&self) -> Ref<'_, CallTrace> {
        self.0.first().unwrap().call_trace.borrow()
    }
}

#[derive(Clone, Default, Debug)]
pub struct CheatedData {
    pub block_number: Option<Felt252>,
    pub block_timestamp: Option<Felt252>,
    pub caller_address: Option<ContractAddress>,
    pub sequencer_address: Option<ContractAddress>,
    pub tx_info: Option<TxInfoMock>,
}

pub struct TraceData {
    pub current_call_stack: NotEmptyCallStack,
    pub is_vm_trace_needed: bool,
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
    pub mocked_functions:
        HashMap<ContractAddress, HashMap<EntryPointSelector, CheatStatus<Vec<StarkFelt>>>>,
    pub spoofed_contracts: HashMap<ContractAddress, CheatStatus<TxInfoMock>>,
    pub global_spoof: Option<(TxInfoMock, CheatSpan)>,
    pub replaced_bytecode_contracts: HashMap<ContractAddress, ClassHash>,
    pub spies: Vec<SpyTarget>,
    pub detected_events: Vec<Event>,
    pub deploy_salt_base: u32,
    pub block_info: BlockInfo,
    pub trace_data: TraceData,
}

impl Default for CheatnetState {
    fn default() -> Self {
        let mut test_code_entry_point = build_test_entry_point();
        test_code_entry_point.class_hash = Some(class_hash!(TEST_CONTRACT_CLASS_HASH));
        let test_call = Rc::new(RefCell::new(CallTrace {
            entry_point: test_code_entry_point,
            ..CallTrace::default_successful_call()
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
            replaced_bytecode_contracts: Default::default(),
            global_spoof: None,
            spies: vec![],
            detected_events: vec![],
            deploy_salt_base: 0,
            block_info: SerializableBlockInfo::default().into(),
            trace_data: TraceData {
                current_call_stack: NotEmptyCallStack::from(test_call),
                is_vm_trace_needed: false,
            },
        }
    }
}

impl CheatnetState {
    #[must_use]
    pub fn create_cheated_data(&self, contract_address: &ContractAddress) -> CheatedData {
        CheatedData {
            block_number: self.get_cheated_block_number(contract_address),
            block_timestamp: self.get_cheated_block_timestamp(contract_address),
            caller_address: self.get_cheated_caller_address(contract_address),
            sequencer_address: self.get_cheated_sequencer_address(contract_address),
            tx_info: self.get_cheated_tx_info(contract_address),
        }
    }

    pub fn get_cheated_data(&mut self, contract_address: &ContractAddress) -> CheatedData {
        let current_call_stack = &mut self.trace_data.current_call_stack;

        // case of cheating the test address itself
        if current_call_stack.size() == 1 {
            self.create_cheated_data(contract_address)
            // do not update the cheats, as the test address cannot be called from the outside
        } else {
            current_call_stack.top_cheated_data()
        }
    }

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

    pub fn update_cheats(&mut self, address: &ContractAddress) {
        update_cheat_for_contract(&self.global_roll, &mut self.rolled_contracts, address);
        update_cheat_for_contract(&self.global_warp, &mut self.warped_contracts, address);
        update_cheat_for_contract(&self.global_prank, &mut self.pranked_contracts, address);
        update_cheat_for_contract(&self.global_elect, &mut self.elected_contracts, address);
        update_cheat_for_contract(&self.global_spoof, &mut self.spoofed_contracts, address);
    }
}

impl TraceData {
    pub fn enter_nested_call(
        &mut self,
        entry_point: CallEntryPoint,
        resources_used_before_call: ExecutionResources,
        cheated_data: CheatedData,
    ) {
        let new_call = Rc::new(RefCell::new(CallTrace {
            entry_point,
            ..CallTrace::default_successful_call()
        }));
        let current_call = self.current_call_stack.top();

        current_call
            .borrow_mut()
            .nested_calls
            .push(CallTraceNode::EntryPointCall(new_call.clone()));

        self.current_call_stack
            .push(new_call, resources_used_before_call, cheated_data);
    }

    pub fn set_class_hash_for_current_call(&mut self, class_hash: ClassHash) {
        let current_call = self.current_call_stack.top();
        current_call.borrow_mut().entry_point.class_hash = Some(class_hash);
    }

    pub fn exit_nested_call(
        &mut self,
        resources_used_after_call: &ExecutionResources,
        used_syscalls: SyscallCounter,
        result: CallResult,
        l2_to_l1_messages: &[OrderedL2ToL1Message],
        vm_trace: Option<Vec<TraceEntry>>,
    ) {
        let CallStackElement {
            resources_used_before_call,
            call_trace: last_call,
            ..
        } = self.current_call_stack.pop();

        let mut last_call = last_call.borrow_mut();
        last_call.used_execution_resources =
            resources_used_after_call - &resources_used_before_call;
        last_call.used_syscalls = used_syscalls;

        last_call.used_l1_resources.l2_l1_message_sizes = l2_to_l1_messages
            .iter()
            .map(|ordered_message| ordered_message.message.payload.0.len())
            .collect();

        last_call.result = result;
        last_call.vm_trace = vm_trace;
    }

    pub fn add_deploy_without_constructor_node(&mut self) {
        let current_call = self.current_call_stack.top();

        current_call
            .borrow_mut()
            .nested_calls
            .push(CallTraceNode::DeployWithoutConstructor);
    }
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
