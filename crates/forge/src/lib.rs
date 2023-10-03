use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::{anyhow, Context, Result};
use ark_std::iterable::Iterable;
use assert_fs::fixture::{FileTouch, PathChild, PathCopy};
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use running::blocking_run_from_test;
use serde::Deserialize;
use std::sync::Arc;
use test_case_summary::TestCaseSummary;
use tokio::task;
use tokio_util::sync::CancellationToken;

use cairo_lang_runner::SierraCasmRunner;
use cairo_lang_sierra::ids::ConcreteTypeId;
use cairo_lang_sierra::program::{Function, Program};
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use once_cell::sync::Lazy;
use rand::{thread_rng, RngCore};
use smol_str::SmolStr;
use walkdir::WalkDir;

use crate::fuzzer::RandomFuzzer;
use crate::running::run_from_test_case;
use crate::scarb::{ForgeConfig, ForkTarget, StarknetContractArtifacts};
pub use crate::test_crate_summary::TestCrateSummary;

use test_collector::{collect_tests, LinkedLibrary, TestCase};

pub mod pretty_printing;
pub mod scarb;
pub mod test_case_summary;

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
    ) -> Arc<Self> {
        Arc::new(Self {
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
        })
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

struct TestsFromCrate {
    sierra_program: Program,
    test_cases: Vec<TestCase>,
    test_crate_type: TestCrateType,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TestCrateType {
    /// Tests collected from the package
    Lib,
    /// Tests collected from the tests folder
    Tests,
}

struct TestCrate {
    crate_root: Utf8PathBuf,
    crate_name: String,
    crate_type: TestCrateType,
}

pub struct RunnerParams {
    corelib_path: Utf8PathBuf,
    contracts: HashMap<String, StarknetContractArtifacts>,
    predeployed_contracts: Utf8PathBuf,
    environment_variables: HashMap<String, String>,
}

impl RunnerParams {
    #[must_use]
    pub fn new(
        corelib_path: Utf8PathBuf,
        contracts: HashMap<String, StarknetContractArtifacts>,
        predeployed_contracts: Utf8PathBuf,
        environment_variables: HashMap<String, String>,
    ) -> Arc<Self> {
        Arc::new(Self {
            corelib_path,
            contracts,
            predeployed_contracts,
            environment_variables,
        })
    }
}

fn collect_tests_from_package(
    package_path: &Utf8PathBuf,
    package_name: &str,
    package_source_dir_path: &Utf8PathBuf,
    linked_libraries: &[LinkedLibrary],
    corelib_path: &Utf8PathBuf,
    runner_config: &RunnerConfig,
) -> Result<Vec<TestsFromCrate>> {
    let tests_dir_path = package_path.join("tests");
    let maybe_tests_tmp_dir = if tests_dir_path.try_exists()? {
        Some(pack_tests_into_one_file(package_path)?)
    } else {
        None
    };

    let mut all_test_roots = vec![TestCrate {
        crate_root: package_source_dir_path.clone(),
        crate_name: package_name.to_string(),
        crate_type: TestCrateType::Lib,
    }];

    if let Some(tests_tmp_dir) = &maybe_tests_tmp_dir {
        let tests_tmp_dir_path = Utf8PathBuf::from_path_buf(tests_tmp_dir.to_path_buf())
            .map_err(|_| anyhow!("Failed to convert tests temporary directory to Utf8PathBuf"))?;

        all_test_roots.push(TestCrate {
            crate_root: tests_tmp_dir_path.clone(),
            crate_name: "tests".to_string(),
            crate_type: TestCrateType::Tests,
        });
    }

    let tests_from_files = all_test_roots
        .par_iter()
        .map(|test_crate| {
            collect_tests_from_tree(test_crate, linked_libraries, corelib_path, runner_config)
        })
        .collect();

    try_close_tmp_dir(maybe_tests_tmp_dir)?;

    tests_from_files
}

fn pack_tests_into_one_file(package_path: &Utf8PathBuf) -> Result<TempDir> {
    let tests_folder_path = package_path.join("tests");

    let tmp_dir = TempDir::new()?;
    tmp_dir
        .copy_from(&tests_folder_path, &["**/*.cairo"])
        .context("Unable to copy files to temporary directory")?;

    let tests_lib_path = tmp_dir.child("lib.cairo");
    if tests_lib_path.try_exists()? {
        return Ok(tmp_dir);
    }
    tests_lib_path.touch()?;

    let mut content = String::new();
    for entry in WalkDir::new(&tests_folder_path)
        .max_depth(1)
        .sort_by_file_name()
    {
        let entry = entry
            .with_context(|| format!("Failed to read directory at path = {tests_folder_path}"))?;
        let path = entry.path();

        if path.is_file() && path.extension().unwrap_or_default() == "cairo" {
            let mod_name = path
                .strip_prefix(&tests_folder_path)
                .expect("Each test file path should start with package path")
                .to_str()
                .context("Unable to convert test file path to string")?
                .strip_suffix(".cairo")
                .expect("Each test file path should have .cairo extension");

            content.push_str(&format!("mod {mod_name};\n"));
        }
    }

    std::fs::write(tests_lib_path, content).context("Failed to write to tests lib file")?;
    Ok(tmp_dir)
}

fn collect_tests_from_tree(
    test_crate: &TestCrate,
    linked_libraries: &[LinkedLibrary],
    corelib_path: &Utf8PathBuf,
    runner_config: &RunnerConfig,
) -> Result<TestsFromCrate> {
    let (sierra_program, test_cases) = collect_tests(
        test_crate.crate_root.as_str(),
        None,
        &test_crate.crate_name,
        linked_libraries,
        Some(BUILTINS.clone()),
        corelib_path.into(),
    )?;

    let test_cases = if let Some(test_name_filter) = &runner_config.test_name_filter {
        filter_tests_by_name(test_name_filter, runner_config.exact_match, test_cases)
    } else {
        test_cases
    };

    Ok(TestsFromCrate {
        sierra_program,
        test_cases,
        test_crate_type: test_crate.crate_type,
    })
}

fn try_close_tmp_dir(maybe_tmp_dir: Option<TempDir>) -> Result<()> {
    if let Some(tmp_dir) = maybe_tmp_dir {
        let path = tmp_dir.path().to_path_buf();
        tmp_dir.close().with_context(|| {
            anyhow!(
            "Failed to close temporary directory = {} with test files. The files might have not been released from filesystem",
            path.display()
        )
        })?;
    };
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
    package_root: Arc<Utf8PathBuf>,
    package_path: &Utf8PathBuf,
    package_name: &str,
    package_source_dir_path: &Utf8PathBuf,
    linked_libraries: &[LinkedLibrary],
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    cancellation_token: Arc<CancellationToken>,
) -> Result<Vec<TestCrateSummary>> {
    let tests = collect_tests_from_package(
        package_path,
        package_name,
        package_source_dir_path,
        linked_libraries,
        &runner_params.corelib_path.clone(),
        &runner_config,
    )?;

    pretty_printing::print_collected_tests_count(
        tests.iter().map(|tests| tests.test_cases.len()).sum(),
        package_name,
    );

    let mut tests_iterator = tests.into_iter();

    let mut fuzzing_happened = false;
    let mut summaries = vec![];

    for tests_from_crate in tests_iterator.by_ref() {
        let package_root = package_root.clone();
        let (summary, was_fuzzed) = run_tests_from_crate(
            package_root,
            tests_from_crate,
            runner_config.clone(),
            runner_params.clone(),
            cancellation_token.clone(),
        )
        .await?;

        fuzzing_happened |= was_fuzzed;
        summaries.push(summary.clone());
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
async fn run_p_test(
    package_root: Arc<Utf8PathBuf>,
    case: Arc<TestCase>,
    runner: Arc<SierraCasmRunner>,
    runner_params: Arc<RunnerParams>,
    runner_config: Arc<RunnerConfig>,
    args: Vec<ConcreteTypeId>,
    cancellation_token: Arc<CancellationToken>,
) -> Result<(TestCaseSummary, Option<u32>)> {
    let exit_first = runner_config.exit_first;
    if args.is_empty() {
        match blocking_run_from_test(
            package_root,
            runner,
            case.clone(),
            runner_config,
            runner_params,
            vec![],
        )
        .await
        {
            Ok(result) => {
                if exit_first {
                    if let TestCaseSummary::Failed { .. } = &result {
                        cancellation_token.cancel();
                    }
                }
                Ok((result, None))
            }
            Err(e) => Err(e),
        }

        // pretty_printing::print_test_result(&result, None);
        // result
    } else {
        run_with_fuzzing(
            package_root,
            runner_config,
            runner_params,
            runner,
            case,
            &args,
            cancellation_token,
        )
        .await
    }
}
//TODO rename
async fn run_tests_from_crate(
    package_root: Arc<Utf8PathBuf>,
    tests: TestsFromCrate,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    cancellation_token: Arc<CancellationToken>,
) -> Result<(TestCrateSummary, bool)> {
    let runner = Arc::new(
        SierraCasmRunner::new(
            tests.sierra_program,
            Some(MetadataComputationConfig::default()),
            OrderedHashMap::default(),
        )
        .context("Failed setting up runner.")?,
    );

    pretty_printing::print_running_tests(tests.test_crate_type, tests.test_cases.len());

    let mut was_fuzzed = false;
    let mut tasks = vec![];
    let test_cases = Arc::new(tests.test_cases);
    test_cases.iter().for_each(|case| {
        let case_name = case.name.as_str();

        let function = runner.find_function(case_name).unwrap();
        let args = function_args(function, &BUILTINS);

        let task = task::spawn({
            let package_root = package_root.clone();
            let case = Arc::new(case.clone());
            let c = case.clone();
            let runner = runner.clone();
            let args: Vec<ConcreteTypeId> = args.into_iter().cloned().collect();
            let cancellation_token = cancellation_token.clone();
            let runner_params = runner_params.clone();
            let runner_config = runner_config.clone();
            async move {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        // The token was cancelled
                       Ok((TestCaseSummary::skipped(&c), None))
                    },
                    result = run_p_test(package_root.clone() ,case, runner,runner_params, runner_config, args, cancellation_token.clone()) => {
                        result
                    },
                }
            }
        });

        tasks.push(task);
    });

    let mut results = vec![];

    for thread in tasks {
        let (result, runs) = thread.await.unwrap()?;
        if runs.is_some() {
            was_fuzzed = true;
        }

        pretty_printing::print_test_result(&result, runs);
        results.push((result, runs));
    }

    Ok((
        TestCrateSummary {
            test_case_summaries: results
                .iter()
                .map(|(result, _runs)| result.clone())
                .collect(),

            runner_exit_status: RunnerStatus::Default,
            test_crate_type: tests.test_crate_type,
        },
        was_fuzzed,
    ))
}

async fn run_with_fuzzing(
    package_root: Arc<Utf8PathBuf>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    runner: Arc<SierraCasmRunner>,
    case: Arc<TestCase>,
    args: &Vec<ConcreteTypeId>,
    cancellation_token: Arc<CancellationToken>,
) -> Result<(TestCaseSummary, Option<u32>)> {
    let token = CancellationToken::new();
    let token = Arc::new(token.clone());

    let args = args
        .iter()
        .map(|arg| {
            arg.debug_name
                .as_ref()
                .ok_or_else(|| anyhow!("Type {arg:?} does not have a debug name"))
                .map(smol_str::SmolStr::as_str)
        })
        .collect::<Result<Vec<_>>>()?;

    let mut fuzzer =
        RandomFuzzer::create(runner_config.fuzzer_seed, runner_config.fuzzer_runs, &args)?;

    let mut tasks = vec![];

    (1..runner_config.fuzzer_runs + 1).for_each(|_| {
        let args = fuzzer.next_args();
        tasks.push(task::spawn({
            let case = case.clone();
            let package_root =  package_root.clone();
            let runner = runner.clone();
            let args = args.clone();
            let cancellation_token = cancellation_token.clone();
            let cancellation_fuzzing_token = Arc::new(token.clone());
            let runner_params = runner_params.clone();
            let runner_config = runner_config.clone();
            async move {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        // The token was cancelled
                        Ok(TestCaseSummary::Failed { name: "fuzzing".to_string(), run_result: None, msg: None, arguments: vec![] })
                    },
                    _ = cancellation_fuzzing_token.cancelled() => {
                        Ok(TestCaseSummary::Failed { name: "fuzzing".to_string(), run_result: None, msg: None, arguments: vec![] })
                    },
                   result = blocking_run_from_test(
                        package_root,
                        runner,
                        case,
                        runner_config,
                        runner_params,
                        args.clone(),
                    ) => {
                        result
                    },
                }
            }
        }));
    });

    let mut results = vec![];
    for task in tasks {
        let result = match task.await? {
            Ok(result) => result,
            Err(e) => {
                token.cancel();
                return Err(e);
            }
        };

        if let TestCaseSummary::Failed { .. } = &result {
            if results.is_empty() {
                token.cancel();
                if runner_config.exit_first {
                    cancellation_token.cancel();
                };
                results.push(result);
            }
        } else {
            results.push(result);
        }
    }

    let result = results
        .last()
        .expect("Test should always run at least once")
        .clone();
    let runs = u32::try_from(results.len())?;
    Ok((result, Some(runs)))
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

fn filter_tests_by_name(
    test_name_filter: &str,
    exact_match: bool,
    test_cases: Vec<TestCase>,
) -> Vec<TestCase> {
    let mut result = vec![];
    for test in test_cases {
        if exact_match {
            if test.name == test_name_filter {
                result.push(test);
            }
        } else if test.name.contains(test_name_filter) {
            result.push(test);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_collector::ExpectedTestResult;

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
            Arc::new(RunnerConfig {
                test_name_filter: None,
                exact_match: false,
                exit_first: false,
                fork_targets: vec![],
                fuzzer_runs: FUZZER_RUNS_DEFAULT,
                fuzzer_seed: config.fuzzer_seed,
            })
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
            Arc::new(RunnerConfig {
                test_name_filter: None,
                exact_match: false,
                exit_first: true,
                fork_targets: vec![],
                fuzzer_runs: 1234,
                fuzzer_seed: 500,
            })
        );
    }

    #[test]
    fn runner_config_argument_precedence() {
        let config_from_scarb = ForgeConfig {
            exit_first: false,
            fork: vec![],
            fuzzer_runs: Some(1234),
            fuzzer_seed: Some(1000),
        };
        let config = RunnerConfig::new(None, false, true, Some(100), Some(32), &config_from_scarb);
        assert_eq!(
            config,
            Arc::new(RunnerConfig {
                test_name_filter: None,
                exact_match: false,
                exit_first: true,
                fork_targets: vec![],
                fuzzer_runs: 100,
                fuzzer_seed: 32,
            })
        );
    }

    #[test]
    fn collecting_tests() {
        let temp = TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let package_path = Utf8PathBuf::from_path_buf(temp.to_path_buf()).unwrap();

        let tests = pack_tests_into_one_file(&package_path).unwrap();
        let virtual_lib_path = tests.join("lib.cairo");
        let virtual_lib_u8_content = std::fs::read(&virtual_lib_path).unwrap();
        let virtual_lib_content = std::str::from_utf8(&virtual_lib_u8_content).unwrap();

        assert!(virtual_lib_path.try_exists().unwrap());
        assert!(virtual_lib_content.contains("mod contract;"));
        assert!(virtual_lib_content.contains("mod ext_function_test;"));
        assert!(virtual_lib_content.contains("mod test_simple;"));
        assert!(virtual_lib_content.contains("mod without_prefix;"));
    }

    #[test]
    fn filtering_tests() {
        let mocked_tests: Vec<TestCase> = vec![
            TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
            },
            TestCase {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
            },
            TestCase {
                name: "outer::crate2::execute_next_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
            },
        ];

        let filtered = filter_tests_by_name("do", false, mocked_tests.clone());
        assert_eq!(
            filtered,
            vec![TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None
            },]
        );

        let filtered = filter_tests_by_name("run", false, mocked_tests.clone());
        assert_eq!(
            filtered,
            vec![TestCase {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None
            },]
        );

        let filtered = filter_tests_by_name("thing", false, mocked_tests.clone());
        assert_eq!(
            filtered,
            vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None
                },
                TestCase {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None
                },
            ]
        );

        let filtered = filter_tests_by_name("nonexistent", false, mocked_tests.clone());
        assert_eq!(filtered, vec![]);

        let filtered = filter_tests_by_name("", false, mocked_tests);
        assert_eq!(
            filtered,
            vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None
                },
                TestCase {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None
                },
            ]
        );
    }

    #[test]
    fn filtering_tests_uses_whole_path() {
        let mocked_tests: Vec<TestCase> = vec![
            TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
            },
            TestCase {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
            },
            TestCase {
                name: "outer::crate2::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
            },
        ];

        let filtered = filter_tests_by_name("crate2::", false, mocked_tests);
        assert_eq!(
            filtered,
            vec![
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None
                },
                TestCase {
                    name: "outer::crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None,
                },
            ]
        );
    }

    #[test]
    fn filtering_with_exact_match() {
        let mocked_tests: Vec<TestCase> = vec![
            TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
            },
            TestCase {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
            },
            TestCase {
                name: "outer::crate3::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
            },
            TestCase {
                name: "do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
            },
        ];

        let filtered = filter_tests_by_name("", true, mocked_tests.clone());
        assert_eq!(filtered, vec![]);

        let filtered = filter_tests_by_name("thing", true, mocked_tests.clone());
        assert_eq!(filtered, vec![]);

        let filtered = filter_tests_by_name("do_thing", true, mocked_tests.clone());
        assert_eq!(
            filtered,
            vec![TestCase {
                name: "do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None
            },]
        );

        let filtered = filter_tests_by_name("crate1::do_thing", true, mocked_tests.clone());
        assert_eq!(
            filtered,
            vec![TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None
            },]
        );

        let filtered = filter_tests_by_name("crate3::run_other_thing", true, mocked_tests.clone());
        assert_eq!(filtered, vec![]);

        let filtered = filter_tests_by_name("outer::crate3::run_other_thing", true, mocked_tests);
        assert_eq!(
            filtered,
            vec![TestCase {
                name: "outer::crate3::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None
            },]
        );
    }

    #[test]
    fn filtering_tests_works_without_crate_in_test_name() {
        let mocked_tests: Vec<TestCase> = vec![
            TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
            },
            TestCase {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
            },
            TestCase {
                name: "thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
                fork_config: None,
            },
        ];

        let result = filter_tests_by_name("thing", false, mocked_tests);
        assert_eq!(
            result,
            vec![
                TestCase {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None
                },
                TestCase {
                    name: "thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                    fork_config: None
                },
            ]
        );
    }
}
