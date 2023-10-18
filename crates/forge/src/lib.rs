use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::{anyhow, Context, Result};
use ark_std::iterable::Iterable;
use assert_fs::TempDir;
use cairo_felt::Felt252;
use camino::{Utf8Path, Utf8PathBuf};

use futures::StreamExt;
use running::blocking_run_from_test;
use serde::Deserialize;
use tokio::sync::mpsc::{channel, Sender};

use std::sync::Arc;
use test_case_summary::TestCaseSummary;
use tokio::task::{self, JoinHandle};
use tokio_util::sync::CancellationToken;

use cairo_lang_runner::SierraCasmRunner;
use cairo_lang_sierra::ids::ConcreteTypeId;
use cairo_lang_sierra::program::Function;
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use futures::stream::FuturesUnordered;

use once_cell::sync::Lazy;
use rand::{thread_rng, RngCore};
use smol_str::SmolStr;

use crate::fuzzer::RandomFuzzer;
use crate::scarb::{ForgeConfig, ForkTarget, StarknetContractArtifacts};

// pub use crate::collecting::CrateLocation;
pub use crate::test_crate_summary::TestCrateSummary;

use crate::collecting::{
    collect_test_compilation_targets, compile_tests, filter_tests_from_crates, CompiledTestCrate,
};
use test_collector::{FuzzerConfig, LinkedLibrary, TestCase};

pub mod pretty_printing;
pub mod scarb;
pub mod test_case_summary;

mod collecting;
mod fuzzer;
mod running;
mod test_crate_summary;
mod test_execution_syscall_handler;

const FUZZER_RUNS_DEFAULT: u32 = 256;

static BUILTINS: Lazy<Vec<&str>> = Lazy::new(|| {
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
#[derive(Deserialize, Debug, PartialEq)]
pub struct RunnerConfig {
    workspace_root: Utf8PathBuf,
    test_name_filter: Option<String>,
    exact_match: bool,
    exit_first: bool,
    fork_targets: Vec<ForkTarget>,
    fuzzer_runs: u32,
    fuzzer_seed: u64,
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        workspace_root: Utf8PathBuf,
        test_name_filter: Option<String>,
        exact_match: bool,
        exit_first: bool,
        fuzzer_runs: Option<u32>,
        fuzzer_seed: Option<u64>,
        forge_config_from_scarb: &ForgeConfig,
    ) -> Self {
        Self {
            workspace_root,
            test_name_filter,
            exact_match,
            exit_first: forge_config_from_scarb.exit_first || exit_first,
            fork_targets: forge_config_from_scarb.fork.clone(),
            fuzzer_runs: fuzzer_runs
                .or(forge_config_from_scarb.fuzzer_runs)
                .unwrap_or(FUZZER_RUNS_DEFAULT),
            fuzzer_seed: fuzzer_seed
                .or(forge_config_from_scarb.fuzzer_seed)
                .unwrap_or_else(|| thread_rng().next_u64()),
        }
    }
}

pub struct CancellationTokens {
    exit_first: CancellationToken,
    error: CancellationToken,
}

impl CancellationTokens {
    #[must_use]
    pub fn new() -> Self {
        let exit_first = CancellationToken::new();
        let error = CancellationToken::new();
        Self { exit_first, error }
    }
}

impl Default for CancellationTokens {
    fn default() -> Self {
        Self::new()
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

pub struct RunnerParams {
    corelib_path: Utf8PathBuf,
    contracts: HashMap<String, StarknetContractArtifacts>,
    predeployed_contracts: Utf8PathBuf,
    environment_variables: HashMap<String, String>,
    linked_libraries: Vec<LinkedLibrary>,
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CrateLocation {
    /// Main crate in a package
    Lib,
    /// Crate in the `tests/` directory
    Tests,
}

fn try_close_tmp_dir(temp_dir: TempDir) -> Result<()> {
    let path = temp_dir.path().to_path_buf();
    temp_dir.close().with_context(|| {
            anyhow!(
            "Failed to close temporary directory = {} with test files. The files might have not been released from filesystem",
            path.display()
        )
        })?;
    Ok(())
}

/// Run the tests in the package at the given path
///
/// # Arguments
///
/// * `package_path` - Absolute path to the top-level of the Cairo package
/// * `lib_path` - Absolute path to the main file in the package (usually `src/lib.cairo`)
/// * `linked_libraries` - Dependencies needed to run the package at `package_path`
/// * `runner_config` - A configuration of the test runner
/// * `corelib_path` - Absolute path to the Cairo corelib
/// * `contracts` - Map with names of contract used in tests and corresponding sierra and casm artifacts
/// * `predeployed_contracts` - Absolute path to predeployed contracts used by starknet state e.g. account contracts
///

#[allow(clippy::implicit_hasher)]
pub async fn run(
    package_path: &Utf8Path,
    package_name: &str,
    package_source_dir_path: &Utf8Path,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    cancellation_tokens: Arc<CancellationTokens>,
) -> Result<Vec<TestCrateSummary>> {
    let temp_dir = TempDir::new()?;

    let compilation_targets =
        collect_test_compilation_targets(package_path, package_name, package_source_dir_path);
    let compilation_targets = compilation_targets
        .into_iter()
        .map(|ct| ct.ensure_lib_file_exists(&temp_dir))
        .collect::<Result<_>>()?;
    let tests = compile_tests(&compilation_targets, &runner_params)?;
    let tests = filter_tests_from_crates(tests, &runner_config);

    try_close_tmp_dir(temp_dir)?;

    pretty_printing::print_collected_tests_count(
        tests.iter().map(|tests| tests.test_cases.len()).sum(),
        package_name,
    );

    let mut summaries = vec![];

    for compiled_test_crate in tests {
        let compiled_test_crate = Arc::new(compiled_test_crate);
        let runner_config = runner_config.clone();
        let runner_params = runner_params.clone();
        let cancellation_tokens = cancellation_tokens.clone();

        pretty_printing::print_running_tests(
            compiled_test_crate.tests_location,
            compiled_test_crate.test_cases.len(),
        );

        let summary = run_tests_from_crate(
            compiled_test_crate,
            runner_config,
            runner_params,
            cancellation_tokens,
        )
        .await?;

        summaries.push(summary);
    }

    pretty_printing::print_test_summary(&summaries);

    if summaries
        .iter()
        .any(|summary| summary.contained_fuzzed_tests)
    {
        pretty_printing::print_test_seed(runner_config.fuzzer_seed);
    }

    Ok(summaries)
}

async fn run_tests_from_crate(
    tests: Arc<CompiledTestCrate>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    cancellation_tokens: Arc<CancellationTokens>,
) -> Result<TestCrateSummary> {
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

    for case in test_cases.iter() {
        let case_name = case.name.as_str();

        let function = runner.find_function(case_name)?;
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
            cancellation_tokens.clone(),
            &send,
            &send_shut_down,
        ));
    }

    let mut results = vec![];

    while let Some(task) = tasks.next().await {
        let result = task??;

        pretty_printing::print_test_result(&result);

        results.push(result);
    }

    rec.close();

    // Waiting for things to finish shutting down
    drop(send_shut_down);
    let _ = rec_shut_down.recv().await;

    let contained_fuzzed_tests = results.iter().any(|summary| summary.runs().is_some());
    Ok(TestCrateSummary {
        test_case_summaries: results,
        runner_exit_status: RunnerStatus::Default,
        test_crate_type: tests.tests_location,
        contained_fuzzed_tests,
    })
}

#[allow(clippy::too_many_arguments)]
fn choose_test_strategy_and_run(
    args: Vec<ConcreteTypeId>,
    case: Arc<TestCase>,
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    cancellation_tokens: Arc<CancellationTokens>,
    send: &Sender<()>,
    send_shut_down: &Sender<()>,
) -> JoinHandle<Result<TestCaseSummary>> {
    if args.is_empty() {
        run_single_test(
            case,
            runner,
            runner_config,
            runner_params,
            cancellation_tokens,
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
            cancellation_tokens,
            send_shut_down.clone(),
        )
    }
}

fn run_single_test(
    case: Arc<TestCase>,
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    cancellation_tokens: Arc<CancellationTokens>,
    send: Sender<()>,
    send_shut_down: Sender<()>,
) -> JoinHandle<Result<TestCaseSummary>> {
    let exit_first = runner_config.exit_first;
    tokio::task::spawn(async move {
        tokio::select! {
            () = cancellation_tokens.exit_first.cancelled() => {
                // Stop executing all tests because flag --exit-first'
                // has been set and one test FAIL
                Ok(TestCaseSummary::skipped(&case))
            },
            () = cancellation_tokens.error.cancelled() => {
                // Stop executing all tests because
                // one of a test returns Err
                Ok(TestCaseSummary::InterruptedByError {  })
            },

            result = blocking_run_from_test(vec![], case.clone(),runner,  runner_config.clone(), runner_params.clone(), send.clone(), send_shut_down.clone() ) => {
                match result? {
                    Ok(result) => {
                        if exit_first {
                            if let TestCaseSummary::Failed { .. } = &result {
                                cancellation_tokens.exit_first.cancel();
                            }
                        }
                        Ok(result)
                    }
                    Err(e) => {
                        cancellation_tokens.error.cancel();
                        Err(e)
                    }
                }
            }
        }
    })
}

fn run_with_fuzzing(
    args: Vec<ConcreteTypeId>,
    case: Arc<TestCase>,
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    cancellation_tokens: Arc<CancellationTokens>,
    send_shut_down: Sender<()>,
) -> JoinHandle<Result<TestCaseSummary>> {
    tokio::task::spawn(async move {
        let token = CancellationToken::new();
        let (send, mut rec) = channel(1);
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

            tasks.push(run_fuzzing_subtest(
                args,
                case.clone(),
                runner.clone(),
                runner_config.clone(),
                runner_params.clone(),
                cancellation_tokens.clone(),
                token.clone(),
                send.clone(),
                send_shut_down.clone(),
            ));
        }

        let mut results = vec![];
        let mut final_result = None;

        while let Some(task) = tasks.next().await {
            let result = task??;

            results.push(result.clone());
            final_result = Some(result.clone());

            match result {
                TestCaseSummary::Failed { .. } | TestCaseSummary::InterruptedByError {} => {
                    break;
                }
                _ => (),
            }
        }

        rec.close();

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

        match final_result {
            Some(result) => Ok(result.with_runs(runs)),
            None => panic!("Test should always run at least once"),
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn run_fuzzing_subtest(
    args: Vec<Felt252>,
    case: Arc<TestCase>,
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    cancellation_tokens: Arc<CancellationTokens>,
    cancellation_fuzzing_token: CancellationToken,
    send: Sender<()>,
    send_shut_down: Sender<()>,
) -> JoinHandle<Result<TestCaseSummary>> {
    let c = case.clone();
    task::spawn(async move {
        tokio::select! {
            () = cancellation_tokens.error.cancelled() => {
                // Stop executing all tests because
                // one of a test returns Err
                Ok(TestCaseSummary::InterruptedByError {  })
            },
            () = cancellation_tokens.exit_first.cancelled() => {
                // Stop executing all tests because flag --exit-first'
                // has been set and one test FAIL
                Ok(TestCaseSummary::skipped(&c))
            },
            () = cancellation_fuzzing_token.cancelled() => {
                // Stop executing all single fuzzing tests
                // because one of fuzzing test has been FAIL
                Ok(TestCaseSummary::SkippedFuzzing {})

            },
           result = blocking_run_from_test(
                args.clone(),
                case,
                runner,
                runner_config.clone(),
                runner_params.clone(),
                send.clone(),
                send_shut_down.clone()
            ) => {
                match result? {
                    Ok(result) => {
                        if let TestCaseSummary::Failed { .. } = &result {
                            if runner_config.exit_first {
                                cancellation_tokens.exit_first.cancel();
                            } else {
                                cancellation_fuzzing_token.cancel();
                            }
                        }
                        Ok(result)
                    }
                    Err(e) => {
                        cancellation_tokens.error.cancel();
                        Err(e)
                    }
                }
            },
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzer_default_seed() {
        let workspace_root: Utf8PathBuf = Default::default();
        let config = RunnerConfig::new(
            workspace_root.clone(),
            None,
            false,
            false,
            None,
            None,
            &Default::default(),
        );
        let config2 = RunnerConfig::new(
            workspace_root,
            None,
            false,
            false,
            None,
            None,
            &Default::default(),
        );

        assert_ne!(config.fuzzer_seed, 0);
        assert_ne!(config2.fuzzer_seed, 0);
        assert_ne!(config.fuzzer_seed, config2.fuzzer_seed);
    }

    #[test]
    fn runner_config_default_arguments() {
        let workspace_root: Utf8PathBuf = Default::default();
        let config = RunnerConfig::new(
            workspace_root.clone(),
            None,
            false,
            false,
            None,
            None,
            &Default::default(),
        );
        assert_eq!(
            config,
            RunnerConfig {
                workspace_root,
                test_name_filter: None,
                exact_match: false,
                exit_first: false,
                fork_targets: vec![],
                fuzzer_runs: FUZZER_RUNS_DEFAULT,
                fuzzer_seed: config.fuzzer_seed,
            }
        );
    }

    #[test]
    fn runner_config_just_scarb_arguments() {
        let config_from_scarb = ForgeConfig {
            exit_first: true,
            fork: vec![],
            fuzzer_runs: Some(1234),
            fuzzer_seed: Some(500),
        };
        let workspace_root: Utf8PathBuf = Default::default();

        let config = RunnerConfig::new(
            workspace_root.clone(),
            None,
            false,
            false,
            None,
            None,
            &config_from_scarb,
        );
        assert_eq!(
            config,
            RunnerConfig {
                workspace_root,
                test_name_filter: None,
                exact_match: false,
                exit_first: true,
                fork_targets: vec![],
                fuzzer_runs: 1234,
                fuzzer_seed: 500,
            }
        );
    }

    #[test]
    fn runner_config_argument_precedence() {
        let workspace_root: Utf8PathBuf = Default::default();

        let config_from_scarb = ForgeConfig {
            exit_first: false,
            fork: vec![],
            fuzzer_runs: Some(1234),
            fuzzer_seed: Some(1000),
        };
        let config = RunnerConfig::new(
            workspace_root.clone(),
            None,
            false,
            true,
            Some(100),
            Some(32),
            &config_from_scarb,
        );
        assert_eq!(
            config,
            RunnerConfig {
                workspace_root,
                test_name_filter: None,
                exact_match: false,
                exit_first: true,
                fork_targets: vec![],
                fuzzer_runs: 100,
                fuzzer_seed: 32,
            }
        );
    }
}
