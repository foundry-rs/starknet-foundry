use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallResult, CallSuccess,
};
use crate::runtime_extensions::common::sum_syscall_usage;
use crate::state::CheatedData;
use blockifier::blockifier_versioned_constants::VersionedConstants;
use blockifier::execution::call_info::{ExecutionSummary, OrderedEvent, OrderedL2ToL1Message};
use blockifier::execution::entry_point::CallEntryPoint;
use blockifier::execution::syscalls::vm_syscall_utils::SyscallUsageMap;
use cairo_annotations::trace_data::L1Resources;
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use cairo_vm::vm::trace::trace_entry::RelocatedTraceEntry;
use conversions::serde::serialize::{BufferWriter, CairoSerialize};
use starknet_api::core::ClassHash;
use starknet_api::execution_resources::GasVector;
use starknet_api::transaction::fields::GasVectorComputationMode;
use starknet_types_core::felt::Felt;
use std::cell::{OnceCell, Ref, RefCell};
use std::rc::Rc;

#[derive(Debug)]
pub struct TraceData {
    pub current_call_stack: NotEmptyCallStack,
    pub is_vm_trace_needed: bool,
}

#[derive(Debug)]
pub struct NotEmptyCallStack(Vec<CallStackElement>);

#[derive(Clone, Debug)]
struct CallStackElement {
    call_trace: Rc<RefCell<CallTrace>>,
    cheated_data: CheatedData,
}

/// Tree structure representing trace of a call.
#[derive(Debug)]
pub struct CallTrace {
    // only these are serialized
    pub entry_point: CallEntryPoint,
    pub nested_calls: Vec<CallTraceNode>,
    pub result: CallResult,
    // serialize end

    // These also include resources used by internal calls
    pub used_execution_resources: ExecutionResources,
    pub used_l1_resources: L1Resources,
    pub used_syscalls_vm_resources: SyscallUsageMap,
    pub used_syscalls_sierra_gas: SyscallUsageMap,
    pub vm_trace: Option<Vec<RelocatedTraceEntry>>,
    pub gas_consumed: u64,
    pub events: Vec<OrderedEvent>,
    pub signature: Vec<Felt>,

    // This is updated only once after the entire test execution.
    pub gas_report_data: Option<GasReportData>,
}

/// Enum representing a node of a trace of a call.
#[derive(Clone, Debug)]
pub enum CallTraceNode {
    EntryPointCall(Rc<RefCell<CallTrace>>),
    DeployWithoutConstructor,
}

#[derive(Clone, Debug)]
pub struct GasReportData {
    pub execution_summary: ExecutionSummary,
    partial_gas_usage: OnceCell<GasVector>,
}

impl TraceData {
    pub fn enter_nested_call(&mut self, entry_point: CallEntryPoint, cheated_data: CheatedData) {
        let new_call = Rc::new(RefCell::new(CallTrace {
            entry_point,
            ..CallTrace::default_successful_call()
        }));
        let current_call = self.current_call_stack.top();

        current_call
            .borrow_mut()
            .nested_calls
            .push(CallTraceNode::EntryPointCall(new_call.clone()));

        self.current_call_stack.push(new_call, cheated_data);
    }

    pub fn set_class_hash_for_current_call(&mut self, class_hash: ClassHash) {
        let current_call = self.current_call_stack.top();
        current_call.borrow_mut().entry_point.class_hash = Some(class_hash);
    }

    pub fn set_vm_trace_for_current_call(&mut self, vm_trace: Vec<RelocatedTraceEntry>) {
        let current_call = self.current_call_stack.top();
        current_call.borrow_mut().vm_trace = Some(vm_trace);
    }

    pub fn update_current_call_result(&mut self, result: CallResult) {
        let current_call = self.current_call_stack.top();
        current_call.borrow_mut().result = result;
    }

    pub fn clear_current_call_events_and_messages(&mut self) {
        let current_call = self.current_call_stack.top();
        current_call.borrow_mut().events.clear();
        current_call
            .borrow_mut()
            .used_l1_resources
            .l2_l1_message_sizes
            .clear();
    }

    #[expect(clippy::too_many_arguments)]
    pub fn update_current_call(
        &mut self,
        execution_resources: ExecutionResources,
        gas_consumed: u64,
        used_syscalls_vm_resources: SyscallUsageMap,
        used_syscalls_sierra_gas: SyscallUsageMap,
        result: CallResult,
        l2_to_l1_messages: &[OrderedL2ToL1Message],
        signature: Vec<Felt>,
        events: Vec<OrderedEvent>,
    ) {
        let current_call = self.current_call_stack.top();
        let mut current_call = current_call.borrow_mut();

        current_call.used_execution_resources = execution_resources;
        current_call.gas_consumed = gas_consumed;
        current_call.used_syscalls_vm_resources = used_syscalls_vm_resources;
        current_call.used_syscalls_sierra_gas = used_syscalls_sierra_gas;

        current_call.used_l1_resources.l2_l1_message_sizes = l2_to_l1_messages
            .iter()
            .map(|ordered_message| ordered_message.message.payload.0.len())
            .collect();

        current_call.result = result;
        current_call.signature = signature;
        current_call.events = events;
    }

    pub fn exit_nested_call(&mut self) {
        self.current_call_stack.pop();
    }

    pub fn add_deploy_without_constructor_node(&mut self) {
        let current_call = self.current_call_stack.top();

        current_call
            .borrow_mut()
            .nested_calls
            .push(CallTraceNode::DeployWithoutConstructor);
    }
}

impl NotEmptyCallStack {
    pub fn from(elem: Rc<RefCell<CallTrace>>) -> Self {
        NotEmptyCallStack(vec![CallStackElement {
            call_trace: elem,
            cheated_data: CheatedData::default(),
        }])
    }

    pub fn push(&mut self, elem: Rc<RefCell<CallTrace>>, cheated_data: CheatedData) {
        self.0.push(CallStackElement {
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

impl CallTrace {
    pub(crate) fn default_successful_call() -> Self {
        Self {
            entry_point: CallEntryPoint::default(),
            used_execution_resources: ExecutionResources::default(),
            used_l1_resources: L1Resources::default(),
            used_syscalls_vm_resources: SyscallUsageMap::default(),
            used_syscalls_sierra_gas: SyscallUsageMap::default(),
            nested_calls: vec![],
            result: Ok(CallSuccess { ret_data: vec![] }),
            vm_trace: None,
            gas_consumed: u64::default(),
            events: vec![],
            signature: vec![],
            gas_report_data: None,
        }
    }

    #[must_use]
    pub fn get_total_used_syscalls(&self) -> SyscallUsageMap {
        sum_syscall_usage(
            self.used_syscalls_vm_resources.clone(),
            &self.used_syscalls_sierra_gas,
        )
    }
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

impl GasReportData {
    #[must_use]
    pub fn new(execution_summary: ExecutionSummary) -> Self {
        Self {
            execution_summary,
            partial_gas_usage: OnceCell::new(),
        }
    }

    pub fn get_gas(&self) -> GasVector {
        *self.partial_gas_usage.get_or_init(|| {
            self.execution_summary.clone().to_partial_gas_vector(
                VersionedConstants::latest_constants(),
                &GasVectorComputationMode::All,
            )
        })
    }
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
