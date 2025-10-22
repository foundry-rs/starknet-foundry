mod args;
mod component;
mod trace_verbosity;

use cheatnet::trace_data::CallTrace;

pub use args::TraceArgs;
use debugging::ContractsDataStore;
pub use trace_verbosity::TraceVerbosity;

#[must_use]
pub fn build_debugging_trace(
    call_trace: &CallTrace,
    contracts_data_store: &ContractsDataStore,
    trace_args: &TraceArgs,
    test_name: String,
) -> Option<debugging::Trace> {
    let components = trace_args.to_components()?;
    let context = debugging::Context::new(contracts_data_store.clone(), components);
    Some(debugging::Trace::new(call_trace, &context, test_name))
}
