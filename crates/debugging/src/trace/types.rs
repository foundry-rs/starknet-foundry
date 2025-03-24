use crate::trace::collect;
use crate::tree::TreeSerialize;
use blockifier::execution::entry_point::CallType;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallResult;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::CallTrace;
use starknet_api::contract_class::EntryPointType;
use starknet_api::core::ContractAddress;
use starknet_api::transaction::fields::Calldata;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::rc::Rc;

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
    pub entry_point_type: EntryPointType,
    pub calldata: Calldata,
    pub storage_address: StorageAddress,
    pub caller_address: CallerAddress,
    pub call_type: CallType,
    pub nested_calls: Vec<ContractTrace>,
    pub call_result: CallResult,
}

#[derive(Debug, Clone)]
pub struct Selector(pub String);

#[derive(Debug, Clone)]
pub struct TestName(pub String);

#[derive(Debug, Clone)]
pub struct ContractName(pub String);

#[derive(Debug, Clone)]
pub struct StorageAddress(pub ContractAddress);

#[derive(Debug, Clone)]
pub struct CallerAddress(pub ContractAddress);

impl Trace {
    /// Creates a new [`Trace`] from a given `cheatnet` [`CallTrace`] and [`ContractsData`] and a test name.
    pub fn from_call_trace(
        call_trace: &Rc<RefCell<CallTrace>>,
        contracts_data: &ContractsData,
        test_name: String,
    ) -> Self {
        collect::trace(call_trace, contracts_data, test_name)
    }
}

impl Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.serialize())
    }
}
