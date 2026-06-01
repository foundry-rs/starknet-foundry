use crate::backtrace::data::ContractBacktraceDataMapping;
use anyhow::Result;
use camino::Utf8Path;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::EncounteredErrors;
use std::env;

mod data;
mod display;
const BACKTRACE_ENV: &str = "SNFORGE_BACKTRACE";

pub struct TestBacktrace {
    /// Program counters of the panic.
    pub pcs: Vec<usize>,

    /// Sierra-statement -> CASM offset mapping
    pub casm_start_offsets: Vec<usize>,
}

#[must_use]
pub fn add_backtrace_footer(
    message: String,
    contracts_data: &ContractsData,
    encountered_errors: &EncounteredErrors,
) -> String {
    if encountered_errors.is_empty() {
        return message;
    }

    let backtrace = if is_backtrace_enabled() {
        get_backtrace(contracts_data, encountered_errors)
    } else {
        format!("note: run with `{BACKTRACE_ENV}=1` environment variable to display a backtrace")
    };

    format!("{message}\n{backtrace}")
}

#[must_use]
pub fn get_backtrace(
    contracts_data: &ContractsData,
    encountered_errors: &EncounteredErrors,
) -> String {
    let class_hashes = encountered_errors.keys().copied().collect();

    ContractBacktraceDataMapping::new(contracts_data, class_hashes)
        .and_then(|data_mapping| {
            encountered_errors
                .iter()
                .map(|(class_hash, pcs)| data_mapping.render_backtrace(pcs, class_hash))
                .collect::<Result<Vec<_>>>()
                .map(|backtrace| backtrace.join("\n"))
        })
        .unwrap_or_else(|err| format!("failed to create backtrace: {err}"))
}

/// Appends a backtrace footer for a failure.
/// When any contract registered an error, prefers the contract-level backtrace.
/// Otherwise, when the panic originates in the test body itself, renders a test-level backtrace.
#[must_use]
pub fn add_test_backtrace_footer(
    message: String,
    contracts_data: &ContractsData,
    encountered_errors: &EncounteredErrors,
    test_backtrace: Option<&TestBacktrace>,
    versioned_program_path: &Utf8Path,
    test_name: &str,
) -> String {
    if !encountered_errors.is_empty() {
        return add_backtrace_footer(message, contracts_data, encountered_errors);
    }

    let Some(test_backtrace) = test_backtrace.filter(|bt| !bt.pcs.is_empty()) else {
        return message;
    };

    let backtrace = if is_backtrace_enabled() {
        get_test_backtrace(test_backtrace, versioned_program_path, test_name)
    } else {
        format!("note: run with `{BACKTRACE_ENV}=1` environment variable to display a backtrace")
    };

    format!("{message}\n{backtrace}")
}

#[must_use]
pub fn get_test_backtrace(
    test_backtrace: &TestBacktrace,
    versioned_program_path: &Utf8Path,
    test_name: &str,
) -> String {
    data::render_test_backtrace(
        test_name,
        test_backtrace.casm_start_offsets.clone(),
        versioned_program_path,
        &test_backtrace.pcs,
    )
    .unwrap_or_else(|err| format!("failed to create backtrace: {err}"))
}

#[must_use]
pub fn is_backtrace_enabled() -> bool {
    env::var(BACKTRACE_ENV).is_ok_and(|value| value == "1")
}
