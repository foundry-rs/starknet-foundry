use crate::fuzzer::RandomFuzzer;
use crate::printing::print_test_result;
use crate::running::{run_fuzz_test, run_test};
use crate::test_case_summary::TestCaseSummary;
use crate::test_crate_summary::TestCrateSummary;
use anyhow::{anyhow, Context, Result};
use cairo_lang_runner::SierraCasmRunner;
use cairo_lang_sierra::ids::ConcreteTypeId;
use cairo_lang_sierra::program::{Function, Program};
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use camino::Utf8PathBuf;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use once_cell::sync::Lazy;
use scarb_artifacts::StarknetContractArtifacts;
use smol_str::SmolStr;
use starknet::core::types::BlockId;
use std::collections::HashMap;
use std::sync::Arc;
use test_collector::FuzzerConfig;
use test_collector::TestCase;
use test_collector::{ForkConfig, LinkedLibrary, RawForkParams};
use tokio::sync::mpsc::{channel, Sender};
use tokio::task::JoinHandle;
use url::Url;

pub mod test_case_summary;
pub mod test_crate_summary;

mod fuzzer;
mod printing;
mod running;
mod test_execution_syscall_handler;

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

#[derive(Debug, PartialEq, Clone)]
pub struct ForkTarget {
    name: String,
    params: RawForkParams,
}

impl ForkTarget {
    #[must_use]
    pub fn new(name: String, params: RawForkParams) -> Self {
        Self { name, params }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn params(&self) -> &RawForkParams {
        &self.params
    }
}

/// Configuration of the test runner
#[derive(Debug, PartialEq)]
pub struct RunnerConfig {
    pub workspace_root: Utf8PathBuf,
    pub exit_first: bool,
    pub fork_targets: Vec<ForkTarget>,
    pub fuzzer_runs: u32,
    pub fuzzer_seed: u64,
}

impl RunnerConfig {
    /// Creates a new `RunnerConfig` from given arguments
    ///
    /// # Arguments
    ///
    /// * `test_name_filter` - Used to filter test cases by names
    /// * `exact_match` - Should test names match the `test_name_filter` exactly
    /// * `exit_first` - Should runner exit after first failed test
    #[must_use]
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    pub fn new(
        workspace_root: Utf8PathBuf,
        exit_first: bool,
        fork_targets: Vec<ForkTarget>,
        fuzzer_runs: u32,
        fuzzer_seed: u64,
    ) -> Self {
        Self {
            workspace_root,
            exit_first,
            fork_targets,
            fuzzer_runs,
            fuzzer_seed,
        }
    }
}

pub struct RunnerParams {
    corelib_path: Utf8PathBuf,
    contracts: HashMap<String, StarknetContractArtifacts>,
    predeployed_contracts: Utf8PathBuf,
    environment_variables: HashMap<String, String>,
    linked_libraries: Vec<LinkedLibrary>,
}

impl RunnerParams {
    #[must_use]
    pub fn linked_libraries(&self) -> &Vec<LinkedLibrary> {
        &self.linked_libraries
    }
    #[must_use]
    pub fn corelib_path(&self) -> &Utf8PathBuf {
        &self.corelib_path
    }
}

impl RunnerParams {
    #[must_use]
    pub fn new(
        corelib_path: Utf8PathBuf,
        contracts: HashMap<String, StarknetContractArtifacts>,
        predeployed_contracts: Utf8PathBuf,
        environment_variables: HashMap<String, String>,
        linked_libraries: Vec<LinkedLibrary>,
    ) -> Self {
        Self {
            corelib_path,
            contracts,
            predeployed_contracts,
            environment_variables,
            linked_libraries,
        }
    }
}

/// Exit status of the runner
#[derive(Debug, PartialEq, Clone)]
pub enum RunnerStatus {
    /// Runner exited without problems
    Default,
    /// Some test failed
    TestFailed,
    /// Runner did not run, e.g. when test cases got skipped
    DidNotRun,
}

#[derive(Debug, Clone)]
pub struct ValidatedForkConfig {
    url: Url,
    block_id: BlockId,
}

impl ValidatedForkConfig {
    #[must_use]
    pub fn new(url: Url, block_id: BlockId) -> Self {
        Self { url, block_id }
    }
}

impl TryFrom<RawForkParams> for ValidatedForkConfig {
    type Error = anyhow::Error;

    fn try_from(value: RawForkParams) -> std::result::Result<Self, Self::Error> {
        Ok(ValidatedForkConfig {
            url: value.url.parse()?,
            block_id: value.block_id,
        })
    }
}

impl ForkConfig for ValidatedForkConfig {}

pub type TestCaseRunnable = TestCase<ValidatedForkConfig>;

#[derive(Debug, Clone)]
pub struct TestCrate {
    sierra_program: Program,
    test_cases: Vec<TestCaseRunnable>,
}

impl TestCrate {
    #[must_use]
    pub fn new(sierra_program: Program, test_cases: Vec<TestCaseRunnable>) -> Self {
        Self {
            sierra_program,
            test_cases,
        }
    }
}

pub trait TestCaseFilter {
    fn should_be_run(&self, test_case: &TestCaseRunnable) -> bool;
}

pub enum TestCrateRunResult {
    Ok(TestCrateSummary),
    Interrupted(TestCrateSummary),
}

pub async fn run_tests_from_crate(
    tests: Arc<TestCrate>,
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

    // The second channel serves as a hold point to ensure all tasks complete
    // their shutdown procedures before moving forward (more info: https://tokio.rs/tokio/topics/shutdown)
    let (send_shut_down, mut rec_shut_down) = channel(1);

    for case in test_cases {
        let case_name = case.name.clone();

        if !tests_filter.should_be_run(case) {
            tasks.push(tokio::task::spawn(async {
                Ok(TestCaseSummary::Ignored { name: case_name })
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
            &send,
            &send_shut_down,
        ));
    }

    let mut results = vec![];
    let mut interrupted = false;

    while let Some(task) = tasks.next().await {
        let result = task??;

        print_test_result(&result);

        if let TestCaseSummary::Failed { .. } = result {
            if runner_config.exit_first {
                interrupted = true;
                rec.close();
            }
        }

        results.push(result);
    }

    // Waiting for things to finish shutting down
    drop(send_shut_down);
    let _ = rec_shut_down.recv().await;

    let contained_fuzzed_tests = results.iter().any(|summary| summary.runs().is_some());
    let summary = TestCrateSummary {
        test_case_summaries: results,
        runner_exit_status: RunnerStatus::Default,
        contained_fuzzed_tests,
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
    send: &Sender<()>,
    send_shut_down: &Sender<()>,
) -> JoinHandle<Result<TestCaseSummary>> {
    if args.is_empty() {
        run_test(
            case,
            runner,
            runner_config,
            runner_params,
            send.clone(),
            send_shut_down.clone(),
        )
    } else {
        run_with_fuzzing(
            args,
            case,
            runner,
            runner_config,
            runner_params,
            send.clone(),
            send_shut_down.clone(),
        )
    }
}

fn run_with_fuzzing(
    args: Vec<ConcreteTypeId>,
    case: Arc<TestCaseRunnable>,
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    send: Sender<()>,
    send_shut_down: Sender<()>,
) -> JoinHandle<Result<TestCaseSummary>> {
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
                send_shut_down.clone(),
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

        let final_result = results
            .last()
            .expect("Test should always run at least once");

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

        if let TestCaseSummary::Passed { .. } = final_result {
            // Because we execute tests parallel, it's possible to
            // get Passed after Skipped. To treat fuzzing a test as Passed
            // we have to ensure that all fuzzing subtests Passed
            if runs != fuzzer_runs {
                return Ok(TestCaseSummary::Skipped {});
            };
        };

        Ok(final_result.clone().with_runs(runs))
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
