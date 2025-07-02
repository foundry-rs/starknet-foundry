use crate::contracts_data_store::ContractsDataStore;
use crate::trace::collect::Collector;
use crate::tree::TreeSerialize;
use crate::verbosity::{Detailed, Standard, Verbosity};
use blockifier::execution::entry_point::CallType;
use cheatnet::state::CallTrace;
use starknet_api::contract_class::EntryPointType;
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
    pub contract_name: ContractName,
    pub entry_point_type: Detailed<EntryPointType>,
    pub calldata: Standard<TransformedCalldata>,
    pub contract_address: Detailed<ContractAddress>,
    pub caller_address: Detailed<CallerAddress>,
    pub call_type: Detailed<CallType>,
    pub nested_calls: Vec<ContractTrace>,
    pub call_result: Standard<TransformedCallResult>,
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
    /// Creates a new [`Trace`] from a given `cheatnet` [`CallTrace`], [`ContractsDataStore`], [`Verbosity`] and a test name.
    #[must_use]
    pub fn new(
        call_trace: &CallTrace,
        contracts_data_store: &ContractsDataStore,
        verbosity: Verbosity,
        test_name: String,
    ) -> Self {
        Collector::new(call_trace, contracts_data_store, verbosity).collect_trace(test_name)
    }
}

impl Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.serialize())
    }
}
