use crate::test_filter::TestsFilter;
use anyhow::Result;
use cairo_lang_runner::RunnerError;
use forge_runner::{
    forge_config::ForgeConfig,
    function_args, maybe_generate_coverage, maybe_save_trace_and_profile,
    package_tests::with_config_resolved::TestTargetWithResolvedConfig,
    printing::print_test_result,
    run_for_test_case,
    test_case_summary::{AnyTestCaseSummary, TestCaseSummary},
    test_target_summary::TestTargetSummary,
    TestCaseFilter,
};
use futures::{stream::FuturesUnordered, StreamExt};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc::channel;

#[non_exhaustive]
pub enum TestTargetRunResult {
    Ok(TestTargetSummary),
    Interrupted(TestTargetSummary),
}

pub async fn run_for_test_target(
    tests: TestTargetWithResolvedConfig,
    forge_config: Arc<ForgeConfig>,
    tests_filter: &TestsFilter,
    _package_name: &str,
) -> Result<TestTargetRunResult> {
    let sierra_program = &tests.sierra_program.program;
    let casm_program = tests.casm_program.clone();

    let mut tasks = FuturesUnordered::new();
    // Initiate two channels to manage the `--exit-first` flag.
    // Owing to `cheatnet` fork's utilization of its own Tokio runtime for RPC requests,
    // test execution must occur within a `tokio::spawn_blocking`.
    // As `spawn_blocking` can't be prematurely cancelled (refer: https://dtantsur.github.io/rust-openstack/tokio/task/fn.spawn_blocking.html),
    // a channel is used to signal the task that test processing is no longer necessary.
    let (send, mut rec) = channel(1);

    let type_declarations: HashMap<_, _> = sierra_program
        .type_declarations
        .iter()
        .map(|f| (f.id.id, f))
        .collect();

    for case in tests.test_cases {
        let case_name = case.name.clone();

        // Check if the test case should be excluded
        if tests_filter.is_excluded(&case) {
            continue;
        }

        if !tests_filter.should_be_run(&case) {
            tasks.push(tokio::task::spawn(async {
                // TODO TestCaseType should also be encoded in the test case definition
                Ok(AnyTestCaseSummary::Single(TestCaseSummary::Ignored {
                    name: case_name,
                }))
            }));
            continue;
        };

        let function = sierra_program
            .funcs
            .iter()
            .find(|f| f.id.debug_name.as_ref().unwrap().ends_with(&case_name))
            .ok_or(RunnerError::MissingFunction { suffix: case_name })?;

        let args = function_args(function, &type_declarations);

        let case = Arc::new(case);

        tasks.push(run_for_test_case(
            args,
            case,
            casm_program.clone(),
            forge_config.clone(),
            tests.sierra_program_path.clone(),
            send.clone(),
        ));
    }

    let mut results = vec![];
    let mut saved_trace_data_paths = vec![];
    let mut interrupted = false;

    while let Some(task) = tasks.next().await {
        let result = task??;

        print_test_result(&result, forge_config.output_config.detailed_resources);

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
