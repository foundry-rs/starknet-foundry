use crate::constants::{build_test_entry_point, TEST_CONTRACT_CLASS_HASH};
use crate::forking::state::ForkStateReader;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallResult;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::cheat_execution_info::{
    ExecutionInfoMock, ResourceBounds,
};
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::Event;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spy_messages_to_l1::MessageToL1;
use blockifier::execution::call_info::OrderedL2ToL1Message;
use blockifier::execution::entry_point::CallEntryPoint;
use blockifier::execution::syscalls::hint_processor::SyscallCounter;
use blockifier::state::errors::StateError::UndeclaredClassHash;
use blockifier::{
    execution::contract_class::ContractClass,
    state::state_api::{StateReader, StateResult},
};
use cairo_annotations::trace_data::L1Resources;
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use cairo_vm::vm::trace::trace_entry::RelocatedTraceEntry;
use conversions::serde::deserialize::CairoDeserialize;
use conversions::serde::serialize::{BufferWriter, CairoSerialize};
use conversions::string::TryFromHexStr;
use runtime::starknet::context::SerializableBlockInfo;
use runtime::starknet::state::DictStateReader;
use starknet_api::core::{ChainId, EntryPointSelector};
use starknet_api::transaction::ContractAddressSalt;
use starknet_api::{
    core::{ClassHash, CompiledClassHash, ContractAddress, Nonce},
    state::StorageKey,
};
use starknet_types_core::felt::Felt;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use starknet_api::block::BlockInfo;

// Specifies the duration of the cheat
#[derive(CairoDeserialize, Copy, Clone, Debug, PartialEq, Eq)]
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
    ) -> StateResult<Felt> {
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
            .get_compiled_class(class_hash)
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

impl ExtendedStateReader {
    pub fn get_chain_id(&self) -> anyhow::Result<Option<ChainId>> {
        self.fork_state_reader
            .as_ref()
            .map(ForkStateReader::chain_id)
            .transpose()
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub enum CheatStatus<T> {
    Cheated(T, CheatSpan),
    #[default]
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

    pub fn as_value(&self) -> Option<T>
    where
        T: Clone,
    {
        match self {
            Self::Cheated(value, _span) => Some(value.clone()),
            Self::Uncheated => None,
        }
    }
}

/// Tree structure representing trace of a call.
pub struct CallTrace {
    pub run_with_call_header: bool,
    // only these are serialized
    pub entry_point: CallEntryPoint,
    pub nested_calls: Vec<CallTraceNode>,
    pub result: CallResult,
    // serialize end

    // These also include resources used by internal calls
    pub used_execution_resources: ExecutionResources,
    pub used_l1_resources: L1Resources,
    pub used_syscalls: SyscallCounter,
    pub vm_trace: Option<Vec<RelocatedTraceEntry>>,
}

impl CairoSerialize for CallTrace {
    fn serialize(&self, output: &mut BufferWriter) {
        self.entry_point.serialize(output);

        let visible_calls: Vec<_> = self
            .nested_calls
            .iter()
            .filter_map(CallTraceNode::extract_entry_point_call)
            .collect();

        visible_calls.serialize(output);

        self.result.serialize(output);
    }
}

impl CallTrace {
    fn default_successful_call() -> Self {
        Self {
            run_with_call_header: Default::default(),
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

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct CheatedTxInfo {
    pub version: Option<Felt>,
    pub account_contract_address: Option<Felt>,
    pub max_fee: Option<Felt>,
    pub signature: Option<Vec<Felt>>,
    pub transaction_hash: Option<Felt>,
    pub chain_id: Option<Felt>,
    pub nonce: Option<Felt>,
    pub resource_bounds: Option<Vec<ResourceBounds>>,
    pub tip: Option<Felt>,
    pub paymaster_data: Option<Vec<Felt>>,
    pub nonce_data_availability_mode: Option<Felt>,
    pub fee_data_availability_mode: Option<Felt>,
    pub account_deployment_data: Option<Vec<Felt>>,
}

impl CheatedTxInfo {
    #[must_use]
    pub fn is_mocked(&self) -> bool {
        self != &Default::default()
    }
}

#[derive(Clone, Default, Debug)]
pub struct CheatedData {
    pub block_number: Option<u64>,
    pub block_timestamp: Option<u64>,
    pub caller_address: Option<ContractAddress>,
    pub sequencer_address: Option<ContractAddress>,
    pub tx_info: CheatedTxInfo,
}

pub struct TraceData {
    pub current_call_stack: NotEmptyCallStack,
    pub is_vm_trace_needed: bool,
}

#[derive(Clone)]
pub struct EncounteredError {
    pub pc: usize,
    pub class_hash: ClassHash,
}

pub struct CheatnetState {
    pub cheated_execution_info_contracts: HashMap<ContractAddress, ExecutionInfoMock>,
    pub global_cheated_execution_info: ExecutionInfoMock,

    pub mocked_functions:
        HashMap<ContractAddress, HashMap<EntryPointSelector, CheatStatus<Vec<Felt>>>>,
    pub replaced_bytecode_contracts: HashMap<ContractAddress, ClassHash>,
    pub detected_events: Vec<Event>,
    pub detected_messages_to_l1: Vec<MessageToL1>,
    pub deploy_salt_base: u32,
    pub block_info: BlockInfo,
    pub trace_data: TraceData,
    pub encountered_errors: Vec<EncounteredError>,
}

impl Default for CheatnetState {
    fn default() -> Self {
        let mut test_code_entry_point = build_test_entry_point();
        test_code_entry_point.class_hash =
            Some(TryFromHexStr::try_from_hex_str(TEST_CONTRACT_CLASS_HASH).unwrap());
        let test_call = Rc::new(RefCell::new(CallTrace {
            entry_point: test_code_entry_point,
            run_with_call_header: true,
            ..CallTrace::default_successful_call()
        }));
        Self {
            cheated_execution_info_contracts: Default::default(),
            global_cheated_execution_info: Default::default(),
            mocked_functions: Default::default(),
            replaced_bytecode_contracts: Default::default(),
            detected_events: vec![],
            detected_messages_to_l1: vec![],
            deploy_salt_base: 0,
            block_info: SerializableBlockInfo::default().into(),
            trace_data: TraceData {
                current_call_stack: NotEmptyCallStack::from(test_call),
                is_vm_trace_needed: false,
            },
            encountered_errors: vec![],
        }
    }
}

impl CheatnetState {
    #[must_use]
    pub fn create_cheated_data(&mut self, contract_address: ContractAddress) -> CheatedData {
        let execution_info = self.get_cheated_execution_info_for_contract(contract_address);

        CheatedData {
            block_number: execution_info.block_info.block_number.as_value(),
            block_timestamp: execution_info.block_info.block_timestamp.as_value(),
            caller_address: execution_info.caller_address.as_value(),
            sequencer_address: execution_info.block_info.sequencer_address.as_value(),
            tx_info: CheatedTxInfo {
                version: execution_info.tx_info.version.as_value(),
                account_contract_address: execution_info
                    .tx_info
                    .account_contract_address
                    .as_value(),
                max_fee: execution_info.tx_info.max_fee.as_value(),
                signature: execution_info.tx_info.signature.as_value(),
                transaction_hash: execution_info.tx_info.transaction_hash.as_value(),
                chain_id: execution_info.tx_info.chain_id.as_value(),
                nonce: execution_info.tx_info.nonce.as_value(),
                resource_bounds: execution_info.tx_info.resource_bounds.as_value(),
                tip: execution_info.tx_info.tip.as_value(),
                paymaster_data: execution_info.tx_info.paymaster_data.as_value(),
                nonce_data_availability_mode: execution_info
                    .tx_info
                    .nonce_data_availability_mode
                    .as_value(),
                fee_data_availability_mode: execution_info
                    .tx_info
                    .fee_data_availability_mode
                    .as_value(),
                account_deployment_data: execution_info.tx_info.account_deployment_data.as_value(),
            },
        }
    }

    pub fn get_cheated_data(&mut self, contract_address: ContractAddress) -> CheatedData {
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
        ContractAddressSalt(Felt::from(self.deploy_salt_base))
    }

    #[must_use]
    pub fn get_cheated_block_number(&mut self, address: ContractAddress) -> Option<u64> {
        self.get_cheated_execution_info_for_contract(address)
            .block_info
            .block_number
            .as_value()
    }

    #[must_use]
    pub fn get_cheated_block_timestamp(&mut self, address: ContractAddress) -> Option<u64> {
        self.get_cheated_execution_info_for_contract(address)
            .block_info
            .block_timestamp
            .as_value()
    }

    #[must_use]
    pub fn get_cheated_sequencer_address(
        &mut self,
        address: ContractAddress,
    ) -> Option<ContractAddress> {
        self.get_cheated_execution_info_for_contract(address)
            .block_info
            .sequencer_address
            .as_value()
    }

    #[must_use]
    pub fn get_cheated_caller_address(
        &mut self,
        address: ContractAddress,
    ) -> Option<ContractAddress> {
        self.get_cheated_execution_info_for_contract(address)
            .caller_address
            .as_value()
    }

    pub fn update_cheats(&mut self, address: &ContractAddress) {
        self.progress_cheated_execution_info(*address);
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
            run_with_call_header: false,
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
        vm_trace: Option<Vec<RelocatedTraceEntry>>,
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
