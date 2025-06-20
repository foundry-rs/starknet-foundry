use crate::backtrace::data::ContractBacktraceDataMapping;
use anyhow::Result;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::EncounteredErrors;
use std::env;

mod data;
mod display;
const BACKTRACE_ENV: &str = "SNFORGE_BACKTRACE";

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
        format!(
            "{message}\nnote: run with `{BACKTRACE_ENV}=1` environment variable to display a backtrace"
        )
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

#[must_use]
pub fn is_backtrace_enabled() -> bool {
    env::var(BACKTRACE_ENV).is_ok_and(|value| value == "1")
}
