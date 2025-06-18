use cheatnet::forking::data::ForksData;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::CallTrace;
use clap::ValueEnum;

/// Trace verbosity level.
#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum TraceVerbosity {
    /// Display test name, contract name and selector.
    Minimal,
    /// Display test name, contract name, selector, calldata and call result.
    Standard,
    /// Display everything.
    Detailed,
}

impl From<TraceVerbosity> for debugging::Verbosity {
    fn from(verbosity: TraceVerbosity) -> Self {
        match verbosity {
            TraceVerbosity::Minimal => debugging::Verbosity::Minimal,
            TraceVerbosity::Standard => debugging::Verbosity::Standard,
            TraceVerbosity::Detailed => debugging::Verbosity::Detailed,
        }
    }
}

#[must_use]
pub fn build_debugging_trace(
    call_trace: &CallTrace,
    contracts_data: &ContractsData,
    trace_verbosity: Option<TraceVerbosity>,
    test_name: String,
    forks_data: ForksData,
) -> Option<debugging::Trace> {
    let contracts_data_store = debugging::ContractsDataStore::new(contracts_data, forks_data);
    Some(debugging::Trace::new(
        call_trace,
        &contracts_data_store,
        trace_verbosity?.into(),
        test_name,
    ))
}
