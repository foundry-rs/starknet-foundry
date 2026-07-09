use anyhow::Result;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::EncounteredErrors;
use std::env;

use data::TestBacktraceData;
pub use data::{BacktraceAnnotations, LazyContractBacktraceDataMapping, TestAnnotations};

mod data;
mod display;
const BACKTRACE_ENV: &str = "SNFORGE_BACKTRACE";

pub struct TestBacktraceContext {
    pub pcs: Vec<usize>,
    pub casm_start_offsets: Vec<usize>,
}

/// Backtraces-scoped outcome of a test run.
///
/// This mirrors the VM-run outcome, not the test verdict.
/// E.g. `#[should_panic]` test case that panics as expected is still `Panic` here.
pub enum TestBacktraceOutcome {
    /// No panic occurred, so there's no backtrace.
    Success,
    /// Panic occurred, backtrace is captured if enabled.
    Panic(Option<TestBacktraceContext>),
}

pub struct BacktraceSources<'a> {
    pub test_annotations: &'a TestAnnotations,
    pub contract_backtrace_mapping: &'a LazyContractBacktraceDataMapping,
}

impl TestBacktraceOutcome {
    #[must_use]
    pub fn is_panic(&self) -> bool {
        matches!(self, Self::Panic(_))
    }

    #[must_use]
    pub fn context(&self) -> Option<&TestBacktraceContext> {
        match self {
            Self::Panic(ctx) => ctx.as_ref(),
            Self::Success => None,
        }
    }
}

#[must_use]
pub fn add_test_backtrace_footer(
    message: String,
    contracts_data: &ContractsData,
    encountered_errors: &EncounteredErrors,
    test_backtrace: &TestBacktraceOutcome,
    test_name: &str,
    sources: &BacktraceSources,
) -> String {
    // Include hint even if backtrace capture was skipped (due to backtrace being disabled).
    // Note: `is_panic()` is treated as equivalent to non-empty backtrace (>= 1 pc),
    // since `get_reversed_pc_traceback` always injects the current pc.
    let has_backtrace = test_backtrace.is_panic() || !encountered_errors.is_empty();

    if !has_backtrace {
        return message;
    }

    let backtrace = if is_backtrace_enabled() {
        get_backtrace(
            contracts_data,
            encountered_errors,
            test_backtrace.context(),
            test_name,
            sources,
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

#[must_use]
pub fn get_backtrace(
    contracts_data: &ContractsData,
    encountered_errors: &EncounteredErrors,
    test_backtrace: Option<&TestBacktraceContext>,
    test_name: &str,
    sources: &BacktraceSources,
) -> Option<String> {
    let BacktraceSources {
        test_annotations,
        contract_backtrace_mapping,
    } = *sources;

    let mut backtrace_parts = Vec::new();

    if !encountered_errors.is_empty() {
        let contract_part = encountered_errors
            .iter()
            .map(|(class_hash, pcs)| {
                contract_backtrace_mapping.render_backtrace(class_hash, pcs, contracts_data)
            })
            .collect::<Result<Vec<_>>>()
            .map_or_else(
                |err| format!("failed to create contract backtrace: {err}"),
                |backtrace| backtrace.join("\n"),
            );

        backtrace_parts.push(contract_part);
    }

    if let Some(bt) = test_backtrace.filter(|bt| !bt.pcs.is_empty()) {
        let test_part = match test_annotations {
            TestAnnotations::Parsed(annotations) => TestBacktraceData::new(
                test_name.to_owned(),
                annotations,
                bt.casm_start_offsets.clone(),
            )
            .render_backtrace(&bt.pcs)
            .unwrap_or_else(|err| format!("failed to create test backtrace: {err}")),
            TestAnnotations::Failed(err) => format!("failed to create test backtrace: {err}"),
            // In practice, this should never happen as debug info is prerequisite checked by `check_backtrace_compatibility`
            TestAnnotations::Missing => {
                "failed to create test backtrace: debug info not found".to_string()
            }
        };

        backtrace_parts.push(test_part);
    }

    (!backtrace_parts.is_empty()).then(|| {
        let body = backtrace_parts.join("\n");
        format!("stack backtrace:\n{body}")
    })
}

#[must_use]
pub fn is_backtrace_enabled() -> bool {
    env::var(BACKTRACE_ENV).is_ok_and(|value| value == "1")
}
