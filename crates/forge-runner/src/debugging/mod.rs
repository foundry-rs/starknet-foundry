mod args;
mod component;
mod trace_verbosity;

use cheatnet::forking::data::ForkData;
use cheatnet::predeployment::abi::predeployed_contracts_debugging_data;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::trace_data::CallTrace;
use debugging::ContractsDataStore;

pub use args::TraceArgs;
pub use trace_verbosity::TraceVerbosity;

#[must_use]
pub fn build_debugging_trace(
    call_trace: &CallTrace,
    contracts_data: &ContractsData,
    trace_args: &TraceArgs,
    test_name: String,
    fork_data: &ForkData,
) -> Option<debugging::Trace> {
    let components = trace_args.to_components()?;
    let is_fork = !fork_data.abi.is_empty() || !fork_data.selectors.is_empty();
    let store = if is_fork {
        ContractsDataStore::new(contracts_data, fork_data)
    } else {
        ContractsDataStore::from(predeployed_contracts_debugging_data())
            .merge(ContractsDataStore::new(contracts_data, fork_data))
    };
    let context = debugging::Context::new(store, components);
    Some(debugging::Trace::new(call_trace, &context, test_name))
}
