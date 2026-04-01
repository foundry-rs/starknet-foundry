use anyhow::Result;
use forge_runner::filtering::{ExcludeReason, FilterResult, TestCaseFilter};
use forge_runner::messages::TestResultMessage;
use forge_runner::{
    forge_config::ForgeConfig,
    maybe_generate_coverage, maybe_save_trace_and_profile,
    package_tests::with_config_resolved::TestTargetWithResolvedConfig,
    run_for_test_case,
    test_case_summary::{AnyTestCaseSummary, TestCaseSummary},
    test_target_summary::TestTargetSummary,
};
use foundry_ui::UI;
use futures::{StreamExt, stream::FuturesUnordered};
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender, channel};

/// Shared cancellation channel for `--exit-first`.
///
/// The channel is created at the workspace level and shared across all packages,
/// so failure in one package immediately interrupts test cases in all subsequent packages.
pub struct ExitFirstChannel {
    sender: Sender<()>,
    receiver: Receiver<()>,
}

impl Default for ExitFirstChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl ExitFirstChannel {
    #[must_use]
    pub fn new() -> Self {
        let (sender, receiver) = channel(1);
        Self { sender, receiver }
    }

    #[must_use]
    pub fn sender(&self) -> Sender<()> {
        self.sender.clone()
    }

    pub fn close(&mut self) {
        self.receiver.close();
    }
}

#[non_exhaustive]
pub enum TestTargetRunResult {
    Ok(TestTargetSummary),
    Interrupted(TestTargetSummary),
}

#[tracing::instrument(skip_all, level = "debug")]
pub async fn run_for_test_target(
    tests: TestTargetWithResolvedConfig,
    forge_config: Arc<ForgeConfig>,
    tests_filter: &impl TestCaseFilter,
    ui: Arc<UI>,
    exit_first_channel: &mut ExitFirstChannel,
) -> Result<TestTargetRunResult> {
    let casm_program = tests.casm_program.clone();

    let mut tasks = FuturesUnordered::new();

    for case in tests.test_cases {
        let case_name = case.name.clone();
        let filter_result = tests_filter.filter(&case);

        match filter_result {
            FilterResult::Excluded(reason) => match reason {
                ExcludeReason::ExcludedFromPartition => {
                    tasks.push(tokio::task::spawn(async {
                        Ok(AnyTestCaseSummary::Single(
                            TestCaseSummary::ExcludedFromPartition {},
                        ))
                    }));
                }
                ExcludeReason::Ignored => {
                    tasks.push(tokio::task::spawn(async {
                        Ok(AnyTestCaseSummary::Single(TestCaseSummary::Ignored {
                            name: case_name,
                        }))
                    }));
                }
            },
            FilterResult::Included => {
                tasks.push(run_for_test_case(
                    Arc::new(case),
                    casm_program.clone(),
                    tests.hints.clone(),
                    forge_config.clone(),
                    tests.sierra_program_path.clone(),
                    exit_first_channel.sender(),
                ));
            }
        }
    }

    let mut results = vec![];
    let mut saved_trace_data_paths = vec![];
    let mut interrupted = false;
    let deterministic_output = forge_config.test_runner_config.deterministic_output;

    let print_test_result = |result: &AnyTestCaseSummary| {
        let test_result_message = TestResultMessage::new(
            result,
            forge_config.output_config.detailed_resources,
            forge_config.test_runner_config.tracked_resource,
        );
        ui.println(&test_result_message);
    };

    while let Some(task) = tasks.next().await {
        let result = task??;

        // Skip printing; Print all results at once in a sorted order once they are available
        if !deterministic_output && should_print_test_result_message(&result) {
            print_test_result(&result);
        }

        let trace_path = maybe_save_trace_and_profile(
            &result,
            &forge_config.output_config.execution_data_to_save,
        )?;
        if let Some(path) = trace_path {
            saved_trace_data_paths.push(path);
        }

        if result.is_failed() && forge_config.test_runner_config.exit_first {
            interrupted = true;
            exit_first_channel.close();
        }

        results.push(result);
    }

    if deterministic_output {
        let mut sorted_results: Vec<_> = results
            .iter()
            .filter(|r| should_print_test_result_message(r))
            .collect();
        sorted_results.sort_by_key(|r| r.name().unwrap_or(""));
        for result in sorted_results {
            print_test_result(result);
        }
    }

    maybe_generate_coverage(
        &forge_config.output_config.execution_data_to_save,
        &saved_trace_data_paths,
        &ui,
    )?;

    let summary = TestTargetSummary {
        test_case_summaries: results,
    };

    if interrupted {
        Ok(TestTargetRunResult::Interrupted(summary))
    } else {
        Ok(TestTargetRunResult::Ok(summary))
    }
}

fn should_print_test_result_message(result: &AnyTestCaseSummary) -> bool {
    !result.is_interrupted() && !result.is_excluded_from_partition()
}
