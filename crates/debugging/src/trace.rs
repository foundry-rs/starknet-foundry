use crate::tree::node::Node;
use crate::tree::serialize::TreeSerialize;
use blockifier::execution::entry_point::CallType;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallResult;
use cheatnet::state::{CallTrace, CallTraceNode};
use starknet_api::contract_class::EntryPointType;
use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_api::transaction::fields::Calldata;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Trace {
    pub selector: EntryPointSelector,
    pub trace_info: TraceInfo,
}

#[derive(Debug, Clone)]
pub struct TraceInfo {
    pub entry_point_type: EntryPointType,
    pub calldata: Calldata,
    pub storage_address: StorageAddress,
    pub caller_address: CallerAddress,
    pub call_type: CallType,
    pub nested_calls: Vec<Trace>,
    pub call_result: CallResult,
}

/// Wrapper for [`ContractAddress`] to distinguish storage addresses at the type level.
#[derive(Debug, Clone)]
pub struct StorageAddress {
    pub address: ContractAddress,
}

/// Wrapper for [`ContractAddress`] to distinguish caller addresses at the type level.
#[derive(Debug, Clone)]
pub struct CallerAddress {
    pub address: ContractAddress,
}

impl Trace {
    /// Creates a new [`Trace`] from a given `cheatnet` [`CallTrace`].
    pub fn from_call_trace(call_trace: &Rc<RefCell<CallTrace>>) -> Self {
        let call_trace = call_trace.borrow();
        let nested_calls = call_trace
            .nested_calls
            .iter()
            .filter_map(CallTraceNode::extract_entry_point_call)
            .map(Self::from_call_trace)
            .collect();

        let trace_info = TraceInfo {
            entry_point_type: call_trace.entry_point.entry_point_type,
            calldata: call_trace.entry_point.calldata.clone(),
            storage_address: StorageAddress {
                address: call_trace.entry_point.storage_address,
            },
            caller_address: CallerAddress {
                address: call_trace.entry_point.caller_address,
            },
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = Node::create_builder();
        let mut node = Node::new(&mut builder);
        self.serialize(&mut node);
        write!(f, "{}", node.into_string())
    }
}
