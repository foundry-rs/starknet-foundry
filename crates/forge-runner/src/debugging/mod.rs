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
    fork_data: Option<&ForkData>,
    disable_predeployed_contracts: bool,
) -> Option<debugging::Trace> {
    let components = trace_args.to_components()?;
    let empty_fork_data = ForkData::default();
    let fork_data_ref = fork_data.unwrap_or(&empty_fork_data);
    let store = if fork_data.is_some() || disable_predeployed_contracts {
        ContractsDataStore::new(contracts_data, fork_data_ref)
    } else {
        ContractsDataStore::from(predeployed_contracts_debugging_data())
            .merge(ContractsDataStore::new(contracts_data, fork_data_ref))
    };
    let context = debugging::Context::new(store, components);
    Some(debugging::Trace::new(call_trace, &context, test_name))
}
