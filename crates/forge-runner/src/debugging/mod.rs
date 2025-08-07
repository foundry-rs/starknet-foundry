mod args;
mod component;
mod trace_verbosity;

use cheatnet::forking::data::ForkData;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::CallTrace;

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
    let context = debugging::Context::new(contracts_data, fork_data, components);
    Some(debugging::Trace::new(call_trace, &context, test_name))
}
