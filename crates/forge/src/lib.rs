use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::{anyhow, Context, Result};
use ark_std::iterable::Iterable;
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use serde::Deserialize;
use test_case_summary::TestCaseSummary;

use cairo_lang_runner::SierraCasmRunner;
use cairo_lang_sierra::ids::ConcreteTypeId;
use cairo_lang_sierra::program::Function;
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use once_cell::sync::Lazy;
use rand::{thread_rng, RngCore};
use smol_str::SmolStr;

use crate::fuzzer::RandomFuzzer;
use crate::running::run_from_test_case;
use crate::scarb::{ForgeConfig, ForkTarget, StarknetContractArtifacts};

pub use crate::collecting::TestCrateType;
pub use crate::test_crate_summary::TestCrateSummary;

use crate::collecting::{
    collect_test_crates, compile_tests_from_test_crates, filter_tests_from_crates, TestsFromCrate,
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
    pub fn new(
        test_name_filter: Option<String>,
        exact_match: bool,
        exit_first: bool,
        fuzzer_runs: Option<u32>,
        fuzzer_seed: Option<u64>,
        forge_config_from_scarb: &ForgeConfig,
    ) -> Self {
        Self {
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
pub fn run(
    package_root: &Utf8PathBuf,
    package_path: &Utf8PathBuf,
    package_name: &str,
    package_source_dir_path: &Utf8PathBuf,
    runner_config: &RunnerConfig,
    runner_params: &RunnerParams,
) -> Result<Vec<TestCrateSummary>> {
    let temp_dir = TempDir::new()?;

    let test_crates = collect_test_crates(
        package_path,
        package_name,
        package_source_dir_path,
        &temp_dir,
    )?;
    let tests = compile_tests_from_test_crates(&test_crates, runner_params)?;
    let tests = filter_tests_from_crates(tests, runner_config);

    try_close_tmp_dir(temp_dir)?;

    pretty_printing::print_collected_tests_count(
        tests.iter().map(|tests| tests.test_cases.len()).sum(),
        package_name,
    );

    let mut tests_iterator = tests.into_iter();

    let mut fuzzing_happened = false;
    let mut summaries = vec![];

    for tests_from_crate in tests_iterator.by_ref() {
        let (summary, was_fuzzed) =
            run_tests_from_crate(package_root, tests_from_crate, runner_config, runner_params)?;

        fuzzing_happened |= was_fuzzed;

        summaries.push(summary.clone());
        if summary.runner_exit_status == RunnerStatus::TestFailed {
            break;
        }
    }

    for tests_from_file in tests_iterator {
        let skipped: Vec<TestCaseSummary> = tests_from_file
            .test_cases
            .iter()
            .map(TestCaseSummary::skipped)
            .collect();

        for test_case_summary in &skipped {
            pretty_printing::print_test_result(test_case_summary, None);
        }

        let file_summary = TestCrateSummary {
            test_case_summaries: skipped,
            runner_exit_status: RunnerStatus::DidNotRun,
            test_crate_type: tests_from_file.test_crate_type,
        };
        summaries.push(file_summary);
    }

    pretty_printing::print_test_summary(&summaries);
    if fuzzing_happened {
        pretty_printing::print_test_seed(runner_config.fuzzer_seed);
    }

    Ok(summaries)
}

fn run_tests_from_crate(
    package_root: &Utf8PathBuf,
    tests: TestsFromCrate,
    runner_config: &RunnerConfig,
    runner_params: &RunnerParams,
) -> Result<(TestCrateSummary, bool)> {
    let runner = SierraCasmRunner::new(
        tests.sierra_program,
        Some(MetadataComputationConfig::default()),
        OrderedHashMap::default(),
    )
    .context("Failed setting up runner.")?;

    pretty_printing::print_running_tests(tests.test_crate_type, tests.test_cases.len());

    let mut was_fuzzed = false;
    let mut results = vec![];

    for (i, case) in tests.test_cases.iter().enumerate() {
        let case_name = case.name.as_str();
        let function = runner.find_function(case_name)?;
        let args = function_args(function, &BUILTINS);

        let result = if args.is_empty() {
            let result = run_from_test_case(
                package_root,
                &runner,
                case,
                runner_config.fork_targets.as_ref(),
                &runner_params.contracts,
                &runner_params.predeployed_contracts,
                vec![],
                &runner_params.environment_variables,
            )?;
            pretty_printing::print_test_result(&result, None);

            result
        } else {
            was_fuzzed = true;
            let (result, runs) = run_with_fuzzing(
                package_root,
                runner_config,
                runner_params,
                &runner,
                case,
                &args,
            )?;
            pretty_printing::print_test_result(&result, Some(runs));

            result
        };

        results.push(result.clone());

        if runner_config.exit_first {
            if let TestCaseSummary::Failed { .. } = result {
                for case in &tests.test_cases[i + 1..] {
                    let skipped_result = TestCaseSummary::skipped(case);
                    pretty_printing::print_test_result(&skipped_result, None);
                    results.push(skipped_result);
                }
                return Ok((
                    TestCrateSummary {
                        test_case_summaries: results,
                        runner_exit_status: RunnerStatus::TestFailed,
                        test_crate_type: tests.test_crate_type,
                    },
                    was_fuzzed,
                ));
            }
        }
    }
    Ok((
        TestCrateSummary {
            test_case_summaries: results,
            runner_exit_status: RunnerStatus::Default,
            test_crate_type: tests.test_crate_type,
        },
        was_fuzzed,
    ))
}

fn run_with_fuzzing(
    package_root: &Utf8PathBuf,
    runner_config: &RunnerConfig,
    runner_params: &RunnerParams,
    runner: &SierraCasmRunner,
    case: &TestCase,
    args: &Vec<&ConcreteTypeId>,
) -> Result<(TestCaseSummary, u32)> {
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

    let mut results = vec![];

    for _ in 1..=fuzzer_runs {
        let args = fuzzer.next_args();

        let result = run_from_test_case(
            package_root,
            runner,
            case,
            runner_config.fork_targets.as_ref(),
            &runner_params.contracts,
            &runner_params.predeployed_contracts,
            args.clone(),
            &runner_params.environment_variables,
        )?;
        results.push(result.clone());

        if let TestCaseSummary::Failed { .. } = result {
            // Fuzz failed
            break;
        }
    }

    let result = results
        .last()
        .expect("Test should always run at least once")
        .clone();
    let runs = u32::try_from(results.len())?;
    Ok((result, runs))
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
        let config = RunnerConfig::new(None, false, false, None, None, &Default::default());
        let config2 = RunnerConfig::new(None, false, false, None, None, &Default::default());

        assert_ne!(config.fuzzer_seed, 0);
        assert_ne!(config2.fuzzer_seed, 0);
        assert_ne!(config.fuzzer_seed, config2.fuzzer_seed);
    }

    #[test]
    fn runner_config_default_arguments() {
        let config = RunnerConfig::new(None, false, false, None, None, &Default::default());
        assert_eq!(
            config,
            RunnerConfig {
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
        let config = RunnerConfig::new(None, false, false, None, None, &config_from_scarb);
        assert_eq!(
            config,
            RunnerConfig {
                test_name_filter: None,
                exact_match: false,
                exit_first: true,
                fork_targets: vec![],
                fuzzer_runs: 1234,
                fuzzer_seed: 500,
            }
        );
    }
}
