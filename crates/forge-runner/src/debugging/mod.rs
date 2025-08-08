mod component;
mod trace_verbosity;

use cheatnet::forking::data::ForkData;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::CallTrace;

pub use trace_verbosity::TraceVerbosity;

#[must_use]
pub fn build_debugging_trace(
    call_trace: &CallTrace,
    contracts_data: &ContractsData,
    trace_verbosity: Option<TraceVerbosity>,
    test_name: String,
    fork_data: &ForkData,
) -> Option<debugging::Trace> {
    let contracts_data_store = debugging::ContractsDataStore::new(contracts_data, fork_data);
    Some(debugging::Trace::new(
        call_trace,
        &contracts_data_store,
        &trace_verbosity?.to_components(),
        test_name,
    ))
}
