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
pub fn build_contracts_data_store(
    contracts_data: &ContractsData,
    fork_data: Option<&ForkData>,
    disable_predeployed_contracts: bool,
) -> ContractsDataStore {
    match fork_data {
        Some(fd) => ContractsDataStore::new(contracts_data, fd),
        None if disable_predeployed_contracts => {
            ContractsDataStore::new(contracts_data, &ForkData::default())
        }
        None => ContractsDataStore::from(predeployed_contracts_debugging_data()).merge(
            ContractsDataStore::new(contracts_data, &ForkData::default()),
        ),
    }
}

#[must_use]
pub fn build_debugging_trace(
    call_trace: &CallTrace,
    trace_args: &TraceArgs,
    test_name: String,
    contracts_data_store: ContractsDataStore,
) -> Option<debugging::Trace> {
    let components = trace_args.to_components()?;
    let context = debugging::Context::new(contracts_data_store, components);
    Some(debugging::Trace::new(call_trace, &context, test_name))
}
