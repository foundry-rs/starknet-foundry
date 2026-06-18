use crate::backtrace::data::{ContractBacktraceDataMapping, TestBacktraceData};
use anyhow::Result;
use camino::Utf8Path;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::EncounteredErrors;
use std::env;

mod data;
mod display;
const BACKTRACE_ENV: &str = "SNFORGE_BACKTRACE";

pub struct TestBacktraceContext {
    pub pcs: Vec<usize>,
    pub casm_start_offsets: Vec<usize>,
}

/// Appends a backtrace footer for a failure.
#[must_use]
pub fn add_test_backtrace_footer(
    message: String,
    contracts_data: &ContractsData,
    encountered_errors: &EncounteredErrors,
    test_backtrace: Option<&TestBacktraceContext>,
    versioned_program_path: &Utf8Path,
    test_name: &str,
) -> String {
    let has_backtrace =
        !encountered_errors.is_empty() || test_backtrace.is_some_and(|bt| !bt.pcs.is_empty());

    if !has_backtrace {
        return message;
    }

    let backtrace = if is_backtrace_enabled() {
        get_backtrace(
            contracts_data,
            encountered_errors,
            test_backtrace,
            versioned_program_path,
            test_name,
        )
    } else {
        Some(format!(
            "note: run with `{BACKTRACE_ENV}=1` environment variable to display a backtrace"
        ))
    };

    if let Some(backtrace) = backtrace {
        format!("{message}\n{backtrace}")
    } else {
        message
    }
}

/// When any contract registered an error, prefers the contract-level backtrace.
/// Otherwise, when the panic originates in the test body itself, renders a test-level backtrace.
#[must_use]
pub fn get_backtrace(
    contracts_data: &ContractsData,
    encountered_errors: &EncounteredErrors,
    test_backtrace: Option<&TestBacktraceContext>,
    versioned_program_path: &Utf8Path,
    test_name: &str,
) -> Option<String> {
    if !encountered_errors.is_empty() {
        let class_hashes = encountered_errors.keys().copied().collect();

        let contract_part = ContractBacktraceDataMapping::new(contracts_data, class_hashes)
            .and_then(|data_mapping| {
                encountered_errors
                    .iter()
                    .map(|(class_hash, pcs)| data_mapping.render_backtrace(pcs, class_hash))
                    .collect::<Result<Vec<_>>>()
                    .map(|backtrace| backtrace.join("\n"))
            })
            .unwrap_or_else(|err| format!("failed to create backtrace: {err}"));

        return Some(contract_part);
    }

    if let Some(bt) = test_backtrace.filter(|bt| !bt.pcs.is_empty()) {
        let test_part = TestBacktraceData::new(
            test_name,
            bt.casm_start_offsets.clone(),
            versioned_program_path,
        )
        .and_then(|data| data.render_backtrace(&bt.pcs))
        .unwrap_or_else(|err| format!("failed to create test backtrace: {err}"));

        return Some(test_part);
    }

    None
}

#[must_use]
pub fn is_backtrace_enabled() -> bool {
    env::var(BACKTRACE_ENV).is_ok_and(|value| value == "1")
}
