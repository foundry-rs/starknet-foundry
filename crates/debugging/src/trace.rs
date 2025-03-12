use crate::tree::TreeSerialize;
use blockifier::execution::entry_point::CallType;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallResult;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::{CallTrace, CallTraceNode};
use starknet_api::contract_class::EntryPointType;
use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_api::transaction::fields::Calldata;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Trace {
    pub selector: EntryPointSelector,
    pub trace_info: TraceInfo,
}

#[derive(Debug, Clone)]
pub struct TraceInfo {
    pub contract_name: Option<ContractName>,
    pub entry_point_type: EntryPointType,
    pub calldata: Calldata,
    pub storage_address: StorageAddress,
    pub caller_address: CallerAddress,
    pub call_type: CallType,
    pub nested_calls: Vec<Trace>,
    pub call_result: CallResult,
}

#[derive(Debug, Clone)]
pub struct ContractName(pub String);

#[derive(Debug, Clone)]
pub struct StorageAddress(pub ContractAddress);

#[derive(Debug, Clone)]
pub struct CallerAddress(pub ContractAddress);

impl Trace {
    /// Creates a new [`Trace`] from a given `cheatnet` [`CallTrace`] and [`ContractsData`].
    pub fn from_call_trace(
        call_trace: &Rc<RefCell<CallTrace>>,
        contracts_data: &ContractsData,
    ) -> Self {
        let call_trace = call_trace.borrow();
        let nested_calls = call_trace
            .nested_calls
            .iter()
            .filter_map(CallTraceNode::extract_entry_point_call)
            .map(|call_trace| Self::from_call_trace(call_trace, contracts_data))
            .collect();

        let trace_info = TraceInfo {
            contract_name: get_contract_name(&call_trace, contracts_data),
            entry_point_type: call_trace.entry_point.entry_point_type,
            calldata: call_trace.entry_point.calldata.clone(),
            storage_address: StorageAddress(call_trace.entry_point.storage_address),
            caller_address: CallerAddress(call_trace.entry_point.caller_address),
            call_type: call_trace.entry_point.call_type,
            nested_calls,
            call_result: call_trace.result.clone(),
        };

        Self {
            selector: call_trace.entry_point.entry_point_selector,
            trace_info,
        }
    }
}

impl Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.serialize())
    }
}

fn get_contract_name(
    call_trace: &CallTrace,
    contracts_data: &ContractsData,
) -> Option<ContractName> {
    contracts_data
        .get_contract_name(
            &call_trace
                .entry_point
                .class_hash
                .expect("this should be set in `fn execute_call_entry_point` in cheatnet"),
        )
        .cloned()
        .map(ContractName)
}
