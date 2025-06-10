use crate::package_tests::with_config_resolved::ResolvedForkConfig;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::CallTrace;
use cheatnet::sync_client::SyncClient;
use clap::ValueEnum;
use debugging::CollectorError;

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
    fork_config: Option<ResolvedForkConfig>,
) -> Option<Result<debugging::Trace, CollectorError>> {
    let client = fork_config.map(|config| SyncClient::new(config.url, config.block_number));
    let mut contracts_data_store = debugging::ContractsDataStore::new(contracts_data, client);
    Some(debugging::Trace::new(
        call_trace,
        &mut contracts_data_store,
        trace_verbosity?.into(),
        test_name,
    ))
}
