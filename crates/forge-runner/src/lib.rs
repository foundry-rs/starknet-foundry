use crate::compiled_runnable::{CompiledTestCrateRunnable, FuzzerConfig, TestCaseRunnable};
use crate::fuzzer::RandomFuzzer;
use crate::printing::print_test_result;
use crate::running::{run_fuzz_test, run_test};
use crate::sierra_casm_runner::SierraCasmRunner;
use crate::test_case_summary::TestCaseSummary;
use crate::test_crate_summary::TestCrateSummary;
use anyhow::{anyhow, Context, Result};

use cairo_lang_sierra::ids::ConcreteTypeId;
use cairo_lang_sierra::program::Function;
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use camino::Utf8PathBuf;

use futures::stream::FuturesUnordered;
use futures::StreamExt;

use once_cell::sync::Lazy;
use scarb_api::StarknetContractArtifacts;
use smol_str::SmolStr;

use std::collections::HashMap;
use std::default::Default;
use std::sync::Arc;
use test_case_summary::{AnyTestCaseSummary, Fuzzing};
use tokio::sync::mpsc::{channel, Sender};
use tokio::task::JoinHandle;

pub mod compiled_runnable;
pub mod expected_result;
pub mod test_case_summary;
pub mod test_crate_summary;

mod fuzzer;
mod gas;
mod printing;
mod running;
mod sierra_casm_runner;
mod sierra_casm_runner_gas;

pub const CACHE_DIR: &str = ".snfoundry_cache";

pub static BUILTINS: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        "Pedersen",
        "RangeCheck",
        "Bitwise",
        "EcOp",
        "Poseidon",
        "SegmentArena",
        "GasBuiltin",
        "System",
    ]
});

/// Configuration of the test runner
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct RunnerConfig {
    pub workspace_root: Utf8PathBuf,
    pub exit_first: bool,
    pub fuzzer_runs: u32,
    pub fuzzer_seed: u64,
}

impl RunnerConfig {
    #[must_use]
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    pub fn new(
        workspace_root: Utf8PathBuf,
        exit_first: bool,
        fuzzer_runs: u32,
        fuzzer_seed: u64,
    ) -> Self {
        Self {
            workspace_root,
            exit_first,
            fuzzer_runs,
            fuzzer_seed,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct RunnerParams {
    contracts: HashMap<String, StarknetContractArtifacts>,
    environment_variables: HashMap<String, String>,
}

impl RunnerParams {
    #[must_use]
    pub fn new(
        contracts: HashMap<String, StarknetContractArtifacts>,
        environment_variables: HashMap<String, String>,
    ) -> Self {
        Self {
            contracts,
            environment_variables,
        }
    }
}

/// Exit status of the runner
#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum RunnerStatus {
    /// Runner exited without problems
    Default,
    /// Some test failed
    TestFailed,
    /// Runner did not run, e.g. when test cases got skipped
    DidNotRun,
}

pub trait TestCaseFilter {
    fn should_be_run(&self, test_case: &TestCaseRunnable) -> bool;
}

#[non_exhaustive]
pub enum TestCrateRunResult {
    Ok(TestCrateSummary),
    Interrupted(TestCrateSummary),
}

pub async fn run_tests_from_crate(
    tests: Arc<CompiledTestCrateRunnable>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    tests_filter: &impl TestCaseFilter,
) -> Result<TestCrateRunResult> {
    let runner = Arc::new(
        SierraCasmRunner::new(
            tests.sierra_program.clone(),
            Some(MetadataComputationConfig::default()),
            OrderedHashMap::default(),
        )
        .context("Failed setting up runner.")?,
    );

    let mut tasks = FuturesUnordered::new();
    let test_cases = &tests.test_cases;
    // Initiate two channels to manage the `--exit-first` flag.
    // Owing to `cheatnet` fork's utilization of its own Tokio runtime for RPC requests,
    // test execution must occur within a `tokio::spawn_blocking`.
    // As `spawn_blocking` can't be prematurely cancelled (refer: https://dtantsur.github.io/rust-openstack/tokio/task/fn.spawn_blocking.html),
    // a channel is used to signal the task that test processing is no longer necessary.
    let (send, mut rec) = channel(1);

    for case in test_cases {
        let case_name = case.name.clone();

        if !tests_filter.should_be_run(case) {
            tasks.push(tokio::task::spawn(async {
                // TODO TestCaseType should also be encoded in the test case definition
                Ok(AnyTestCaseSummary::Single(TestCaseSummary::Ignored {
                    name: case_name,
                }))
            }));
            continue;
        };

        let function = runner.find_function(&case_name)?;
        let args = function_args(function, &BUILTINS);

        let case = Arc::new(case.clone());
        let args: Vec<ConcreteTypeId> = args.into_iter().cloned().collect();
        let runner = runner.clone();

        tasks.push(choose_test_strategy_and_run(
            args,
            case.clone(),
            runner,
            runner_config.clone(),
            runner_params.clone(),
            send.clone(),
        ));
    }

    let mut results = vec![];
    let mut interrupted = false;

    while let Some(task) = tasks.next().await {
        let result = task??;

        print_test_result(&result);

        if result.is_failed() && runner_config.exit_first {
            interrupted = true;
            rec.close();
        }

        results.push(result);
    }

    let summary = TestCrateSummary {
        test_case_summaries: results,
        runner_exit_status: RunnerStatus::Default,
    };

    if interrupted {
        Ok(TestCrateRunResult::Interrupted(summary))
    } else {
        Ok(TestCrateRunResult::Ok(summary))
    }
}

#[allow(clippy::too_many_arguments)]
fn choose_test_strategy_and_run(
    args: Vec<ConcreteTypeId>,
    case: Arc<TestCaseRunnable>,
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    send: Sender<()>,
) -> JoinHandle<Result<AnyTestCaseSummary>> {
    if args.is_empty() {
        tokio::task::spawn(async move {
            let res = run_test(case, runner, runner_config, runner_params, send).await??;
            Ok(AnyTestCaseSummary::Single(res))
        })
    } else {
        tokio::task::spawn(async move {
            let res =
                run_with_fuzzing(args, case, runner, runner_config, runner_params, send).await??;
            Ok(AnyTestCaseSummary::Fuzzing(res))
        })
    }
}

fn run_with_fuzzing(
    args: Vec<ConcreteTypeId>,
    case: Arc<TestCaseRunnable>,
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    send: Sender<()>,
) -> JoinHandle<Result<TestCaseSummary<Fuzzing>>> {
    tokio::task::spawn(async move {
        if send.is_closed() {
            return Ok(TestCaseSummary::Skipped {});
        }

        let (fuzzing_send, mut fuzzing_rec) = channel(1);
        let args = args
            .iter()
            .map(|arg| {
                arg.debug_name
                    .as_ref()
                    .ok_or_else(|| anyhow!("Type {arg:?} does not have a debug name"))
                    .map(SmolStr::as_str)
            })
            .collect::<Result<Vec<_>>>()?;

        let (fuzzer_runs, fuzzer_seed) = match case.fuzzer_config {
            Some(FuzzerConfig {
                fuzzer_runs,
                fuzzer_seed,
            }) => (fuzzer_runs, fuzzer_seed),
            _ => (runner_config.fuzzer_runs, runner_config.fuzzer_seed),
        };
        let mut fuzzer = RandomFuzzer::create(fuzzer_seed, fuzzer_runs, &args)?;

        let mut tasks = FuturesUnordered::new();

        for _ in 1..=fuzzer_runs {
            let args = fuzzer.next_args();

            tasks.push(run_fuzz_test(
                args,
                case.clone(),
                runner.clone(),
                runner_config.clone(),
                runner_params.clone(),
                send.clone(),
                fuzzing_send.clone(),
            ));
        }

        let mut results = vec![];
        while let Some(task) = tasks.next().await {
            let result = task??;

            results.push(result.clone());

            if let TestCaseSummary::Failed { .. } = result {
                fuzzing_rec.close();
                break;
            }
        }

        let runs = u32::try_from(
            results
                .iter()
                .filter(|item| {
                    matches!(
                        item,
                        TestCaseSummary::Passed { .. } | TestCaseSummary::Failed { .. }
                    )
                })
                .count(),
        )?;

        let fuzzing_run_summary: TestCaseSummary<Fuzzing> = TestCaseSummary::from(results);

        if let TestCaseSummary::Passed { .. } = fuzzing_run_summary {
            // Because we execute tests parallel, it's possible to
            // get Passed after Skipped. To treat fuzzing a test as Passed
            // we have to ensure that all fuzzing subtests Passed
            if runs != fuzzer_runs {
                return Ok(TestCaseSummary::Skipped {});
            };
        };

        Ok(fuzzing_run_summary)
    })
}

fn function_args<'a>(function: &'a Function, builtins: &[&str]) -> Vec<&'a ConcreteTypeId> {
    let builtins: Vec<_> = builtins
        .iter()
        .map(|builtin| Some(SmolStr::new(builtin)))
        .collect();

    function
        .signature
        .param_types
        .iter()
        .filter(|pt| !builtins.contains(&pt.debug_name))
        .collect()
}
