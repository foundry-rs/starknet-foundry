use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::{anyhow, Context, Result};
use ark_std::iterable::Iterable;

use camino::{Utf8Path, Utf8PathBuf};

use futures::StreamExt;
use running::{run_fuzz_test, run_test};
use tokio::sync::mpsc::{channel, Sender};

use std::sync::Arc;
use test_case_summary::TestCaseSummary;
use tokio::task::JoinHandle;

use cairo_lang_runner::SierraCasmRunner;
use cairo_lang_sierra::ids::ConcreteTypeId;
use cairo_lang_sierra::program::Function;
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use futures::stream::FuturesUnordered;
use itertools::Itertools;

use once_cell::sync::Lazy;
use smol_str::SmolStr;

use scarb_artifacts::StarknetContractArtifacts;

use crate::fuzzer::RandomFuzzer;
use crate::scarb::config::ForkTarget;

// pub use crate::collecting::CrateLocation;
pub use crate::test_crate_summary::TestCrateSummary;

use crate::collecting::{
    collect_test_compilation_targets, compile_tests, CompiledTestCrate, CompiledTestCrateRaw,
    CompiledTestCrateRunnable, TestCaseRunnable, ValidatedForkConfig,
};
use crate::test_filter::TestsFilter;
use test_collector::{FuzzerConfig, LinkedLibrary, RawForkConfig, RawForkParams, TestCase};

pub mod pretty_printing;
pub mod scarb;
pub mod test_case_summary;
pub mod test_filter;

mod collecting;
mod fuzzer;
mod running;
mod test_crate_summary;
mod test_execution_syscall_handler;

pub const FUZZER_RUNS_DEFAULT: u32 = 256;
pub const CACHE_DIR: &str = ".snfoundry_cache";

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

fn parse_fork_params(raw_fork_params: &RawForkParams) -> Result<ValidatedForkConfig> {
    Ok(ValidatedForkConfig {
        url: raw_fork_params.url.parse()?,
        block_id: raw_fork_params.block_id,
    })
}

fn replace_id_with_params(
    raw_fork_config: RawForkConfig,
    runner_config: &RunnerConfig,
) -> Result<RawForkParams> {
    match raw_fork_config {
        RawForkConfig::Params(raw_fork_params) => Ok(raw_fork_params),
        RawForkConfig::Id(name) => {
            let fork_target_from_runner_config = runner_config
                .fork_targets
                .iter()
                .find(|fork| fork.name == name)
                .ok_or_else(|| {
                    anyhow!("Fork configuration named = {name} not found in the Scarb.toml")
                })?;

            Ok(fork_target_from_runner_config.params.clone())
        }
    }
}

fn to_runnable(
    compiled_test_crate: CompiledTestCrateRaw,
    runner_config: &RunnerConfig,
) -> Result<CompiledTestCrateRunnable> {
    let mut test_cases = vec![];

    for case in compiled_test_crate.test_cases {
        let fork_config = if let Some(fc) = case.fork_config {
            let raw_fork_params = replace_id_with_params(fc, runner_config)?;
            let validated_fork_config = parse_fork_params(&raw_fork_params)?;
            Some(validated_fork_config)
        } else {
            None
        };

        test_cases.push(TestCase {
            name: case.name,
            available_gas: case.available_gas,
            ignored: case.ignored,
            expected_result: case.expected_result,
            fork_config,
            fuzzer_config: case.fuzzer_config,
        });
    }

    Ok(CompiledTestCrate {
        sierra_program: compiled_test_crate.sierra_program,
        test_cases,
        tests_location: compiled_test_crate.tests_location,
    })
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
    tests_filter: &TestsFilter,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
) -> Result<Vec<TestCrateSummary>> {
    let compilation_targets =
        collect_test_compilation_targets(package_path, package_name, package_source_dir_path)?;
    let test_crates = compile_tests(&compilation_targets, &runner_params)?;
    let all_tests: usize = test_crates.iter().map(|tc| tc.test_cases.len()).sum();

    let test_crates = test_crates
        .into_iter()
        .map(|tc| tests_filter.filter_tests(tc))
        .collect_vec();
    let not_filtered: usize = test_crates.iter().map(|tc| tc.test_cases.len()).sum();
    let filtered = all_tests - not_filtered;

    let test_crates = test_crates
        .into_iter()
        .map(|ctc| to_runnable(ctc, &runner_config))
        .collect::<Result<Vec<_>>>()?;

    pretty_printing::print_collected_tests_count(
        test_crates.iter().map(|tests| tests.test_cases.len()).sum(),
        package_name,
    );

    let mut summaries = vec![];

    for compiled_test_crate in test_crates {
        let compiled_test_crate = Arc::new(compiled_test_crate);
        let runner_config = runner_config.clone();
        let runner_params = runner_params.clone();

        pretty_printing::print_running_tests(
            compiled_test_crate.tests_location,
            compiled_test_crate.test_cases.len(),
        );

        let summary = run_tests_from_crate(
            compiled_test_crate,
            runner_config,
            runner_params,
            tests_filter,
        )
        .await?;

        match summary {
            TestCrateRunResult::Ok(summary) => {
                summaries.push(summary);
            }
            TestCrateRunResult::Interrupted(summary) => {
                summaries.push(summary);
                // Handle scenario for --exit-first flag.
                // Because snforge runs test crates one by one synchronously.
                // In case of test FAIL with --exit-first flag stops processing the next crates
                break;
            }
        }
    }

    pretty_printing::print_test_summary(&summaries, filtered);

    if summaries
        .iter()
        .any(|summary| summary.contained_fuzzed_tests)
    {
        pretty_printing::print_test_seed(runner_config.fuzzer_seed);
    }

    Ok(summaries)
}
enum TestCrateRunResult {
    Ok(TestCrateSummary),
    Interrupted(TestCrateSummary),
}

pub trait TestCaseFilter {
    fn should_be_run(&self, test_case: &TestCase<ValidatedForkConfig>) -> bool;
}

async fn run_tests_from_crate(
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

    // The second channel serves as a hold point to ensure all tasks complete
    // their shutdown procedures before moving forward (more info: https://tokio.rs/tokio/topics/shutdown)
    let (send_shut_down, mut rec_shut_down) = channel(1);

    for case in test_cases.iter() {
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

        pretty_printing::print_test_result(&result);

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
        test_crate_type: tests.tests_location,
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

#[cfg(test)]
mod tests {
    use super::*;
    use cairo_lang_sierra::program::Program;
    use starknet::core::types::BlockId;
    use starknet::core::types::BlockTag::Latest;
    use test_collector::ExpectedTestResult;

    #[test]
    fn to_runnable_unparsable_url() {
        let mocked_tests = CompiledTestCrate {
            sierra_program: Program {
                type_declarations: vec![],
                libfunc_declarations: vec![],
                statements: vec![],
                funcs: vec![],
            },
            test_cases: vec![TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                ignored: false,
                expected_result: ExpectedTestResult::Success,
                fork_config: Some(RawForkConfig::Params(RawForkParams {
                    url: "unparsable_url".to_string(),
                    block_id: BlockId::Tag(Latest),
                })),
                fuzzer_config: None,
            }],
            tests_location: CrateLocation::Lib,
        };
        let config = RunnerConfig::new(Default::default(), false, vec![], 256, 12345);

        assert!(to_runnable(mocked_tests, &config).is_err());
    }

    #[test]
    fn to_runnable_non_existent_id() {
        let mocked_tests = CompiledTestCrate {
            sierra_program: Program {
                type_declarations: vec![],
                libfunc_declarations: vec![],
                statements: vec![],
                funcs: vec![],
            },
            test_cases: vec![TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                ignored: false,
                expected_result: ExpectedTestResult::Success,
                fork_config: Some(RawForkConfig::Id("non_existent".to_string())),
                fuzzer_config: None,
            }],
            tests_location: CrateLocation::Lib,
        };
        let config = RunnerConfig::new(
            Default::default(),
            false,
            vec![ForkTarget {
                name: "definitely_non_existing".to_string(),
                params: RawForkParams {
                    url: "https://not_taken.com".to_string(),
                    block_id: BlockId::Number(120),
                },
            }],
            256,
            12345,
        );

        assert!(to_runnable(mocked_tests, &config).is_err());
    }
}
