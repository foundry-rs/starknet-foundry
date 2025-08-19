use crate::Context;
use crate::trace::collect::Collector;
use crate::trace::components::{
    CallResultContainer, CallTypeContainer, CalldataContainer, CallerAddressContainer,
    ContractAddressContainer, ContractNameContainer, EntryPointTypeContainer,
};
use crate::tree::TreeSerialize;
use cheatnet::state::CallTrace;
use starknet_api::core::ContractAddress as ApiContractAddress;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Trace {
    pub test_name: TestName,
    pub nested_calls: Vec<ContractTrace>,
}

#[derive(Debug, Clone)]
pub struct ContractTrace {
    pub selector: Selector,
    pub trace_info: TraceInfo,
}

#[derive(Debug, Clone)]
pub struct TraceInfo {
    pub contract_name: ContractNameContainer,
    pub entry_point_type: EntryPointTypeContainer,
    pub calldata: CalldataContainer,
    pub contract_address: ContractAddressContainer,
    pub caller_address: CallerAddressContainer,
    pub call_type: CallTypeContainer,
    pub nested_calls: Vec<ContractTrace>,
    pub call_result: CallResultContainer,
}

#[derive(Debug, Clone)]
pub struct TransformedCallResult(pub String);

#[derive(Debug, Clone)]
pub struct TransformedCalldata(pub String);

#[derive(Debug, Clone)]
pub struct Selector(pub String);

#[derive(Debug, Clone)]
pub struct TestName(pub String);

#[derive(Debug, Clone)]
pub struct ContractName(pub String);

#[derive(Debug, Clone)]
pub struct ContractAddress(pub ApiContractAddress);

#[derive(Debug, Clone)]
pub struct CallerAddress(pub ApiContractAddress);

impl Trace {
    /// Creates a new [`Trace`] from a given [`Context`] and a test name.
    #[must_use]
    pub fn new(call_trace: &CallTrace, context: &Context, test_name: String) -> Self {
        Collector::new(call_trace, context).collect_trace(test_name)
    }
}

impl Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.serialize())
    }
}
