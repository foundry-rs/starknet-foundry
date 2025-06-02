use anyhow::Result;
use forge_runner::debugging::TraceVerbosity;
use forge_runner::messages::TestResultMessage;
use forge_runner::{
    TestCaseFilter,
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
use tokio::sync::mpsc::channel;

#[non_exhaustive]
pub enum TestTargetRunResult {
    Ok(TestTargetSummary),
    Interrupted(TestTargetSummary),
}

pub async fn run_for_test_target(
    tests: TestTargetWithResolvedConfig,
    forge_config: Arc<ForgeConfig>,
    tests_filter: &impl TestCaseFilter,
    trace_verbosity: Option<TraceVerbosity>,
    ui: &UI,
) -> Result<TestTargetRunResult> {
    let casm_program = tests.casm_program.clone();

    let mut tasks = FuturesUnordered::new();
    // Initiate two channels to manage the `--exit-first` flag.
    // Owing to `cheatnet` fork's utilization of its own Tokio runtime for RPC requests,
    // test execution must occur within a `tokio::spawn_blocking`.
    // As `spawn_blocking` can't be prematurely cancelled (refer: https://dtantsur.github.io/rust-openstack/tokio/task/fn.spawn_blocking.html),
    // a channel is used to signal the task that test processing is no longer necessary.
    let (send, mut rec) = channel(1);

    for case in tests.test_cases {
        let case_name = case.name.clone();

        if !tests_filter.should_be_run(&case) {
            tasks.push(tokio::task::spawn(async {
                // TODO TestCaseType should also be encoded in the test case definition
                Ok(AnyTestCaseSummary::Single(TestCaseSummary::Ignored {
                    name: case_name,
                }))
            }));
            continue;
        }

        let case = Arc::new(case);

        tasks.push(run_for_test_case(
            case,
            casm_program.clone(),
            forge_config.clone(),
            tests.sierra_program_path.clone(),
            send.clone(),
            trace_verbosity,
        ));
    }

    let mut results = vec![];
    let mut saved_trace_data_paths = vec![];
    let mut interrupted = false;

    while let Some(task) = tasks.next().await {
        let result = task??;

        if !result.is_skipped() {
            let test_result_message = TestResultMessage::new(
                &result,
                forge_config.output_config.detailed_resources,
                forge_config.test_runner_config.tracked_resource,
            );
            ui.println(&test_result_message);
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
            rec.close();
        }

        results.push(result);
    }

    maybe_generate_coverage(
        &forge_config.output_config.execution_data_to_save,
        &saved_trace_data_paths,
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
