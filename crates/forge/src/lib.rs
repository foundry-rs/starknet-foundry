use anyhow::{bail, Context, Result};
use ark_std::iterable::Iterable;
use cairo_felt::Felt252;
use camino::{Utf8Path, Utf8PathBuf};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use test_case_summary::TestCaseSummary;
use tokio_util::sync::CancellationToken;

use tokio::task;

use walkdir::WalkDir;

use cairo_lang_runner::SierraCasmRunner;
use cairo_lang_sierra::ids::ConcreteTypeId;
use cairo_lang_sierra::program::{Function, Program};
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use num_bigint::BigUint;
use num_traits::Zero;
use once_cell::sync::Lazy;
use smol_str::SmolStr;

use crate::fuzzer::RandomFuzzer;
use crate::running::run_from_test_case;
use crate::scarb::{ForgeConfig, StarknetContractArtifacts};
pub use crate::test_file_summary::TestFileSummary;
use test_collector::{collect_tests, LinkedLibrary, TestCase};

pub mod pretty_printing;
pub mod scarb;
pub mod test_case_summary;

mod cheatcodes_hint_processor;
mod fuzzer;
mod running;
mod test_file_summary;

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
#[derive(Deserialize, Debug, PartialEq, Default)]
pub struct RunnerConfig {
    test_name_filter: Option<String>,
    exact_match: bool,
    exit_first: bool,
    fuzzer_runs: u32,
    fuzzer_seed: Option<u64>,
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
            fuzzer_runs: fuzzer_runs
                .or(forge_config_from_scarb.fuzzer_runs)
                .unwrap_or(FUZZER_RUNS_DEFAULT),
            fuzzer_seed: fuzzer_seed.or(forge_config_from_scarb.fuzzer_seed),
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

struct TestsFromFile {
    sierra_program: Program,
    test_cases: Vec<TestCase>,
    relative_path: Utf8PathBuf,
}

fn collect_tests_from_package(
    package_path: &Utf8PathBuf,
    package_name: &str,
    lib_path: &Utf8PathBuf,
    linked_libraries: &Option<Vec<LinkedLibrary>>,
    corelib_path: &Utf8PathBuf,
    runner_config: &RunnerConfig,
) -> Result<Vec<TestsFromFile>> {
    let test_files = find_test_files(package_path, lib_path)?;
    test_files
        .par_iter()
        .map(|tf| {
            collect_tests_from_tree(
                tf,
                package_path,
                package_name,
                linked_libraries,
                corelib_path,
                runner_config,
            )
        })
        .collect()
}

fn find_test_files(package_path: &Utf8PathBuf, lib_path: &Utf8PathBuf) -> Result<Vec<Utf8PathBuf>> {
    let mut test_files: Vec<Utf8PathBuf> = vec![lib_path.clone()];
    let tests_folder_path = package_path.join("tests");

    if tests_folder_path.try_exists()? {
        for entry in WalkDir::new(tests_folder_path).sort_by_file_name() {
            let entry = entry
                .with_context(|| format!("Failed to read directory at path = {package_path}"))?;
            let path = entry.path();

            if path.is_file() && path.extension().unwrap_or_default() == "cairo" {
                test_files.push(
                    Utf8Path::from_path(path)
                        .with_context(|| format!("Failed to convert path = {path:?} to utf-8"))?
                        .to_path_buf(),
                );
            }
        }
    }
    Ok(test_files)
}

fn collect_tests_from_tree(
    test_root: &Utf8PathBuf,
    package_path: &Utf8PathBuf,
    package_name: &str,
    linked_libraries: &Option<Vec<LinkedLibrary>>,
    corelib_path: &Utf8PathBuf,
    runner_config: &RunnerConfig,
) -> Result<TestsFromFile> {
    let (sierra_program, tests_configs) = collect_tests(
        test_root.as_str(),
        None,
        package_name,
        linked_libraries.clone(),
        Some(BUILTINS.clone()),
        corelib_path.into(),
    )?;

    let test_cases = if let Some(test_name_filter) = &runner_config.test_name_filter {
        filter_tests_by_name(test_name_filter, runner_config.exact_match, tests_configs)
    } else {
        tests_configs
    };

    let relative_path = test_root.strip_prefix(package_path)?.to_path_buf();

    Ok(TestsFromFile {
        sierra_program,
        test_cases,
        relative_path,
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
#[allow(clippy::implicit_hasher, clippy::too_many_arguments)]
pub async fn run(
    package_path: &Utf8PathBuf,
    package_name: &str,
    lib_path: &Utf8PathBuf,
    linked_libraries: &Option<Vec<LinkedLibrary>>,
    runner_config: &RunnerConfig,
    corelib_path: &Utf8PathBuf,
    contracts: &HashMap<String, StarknetContractArtifacts>,
    predeployed_contracts: &Utf8PathBuf,
    fuzzer_seed: u64,
    cancellation_token: Arc<CancellationToken>,
) -> Result<Vec<TestFileSummary>> {
    let tests = collect_tests_from_package(
        package_path,
        package_name,
        lib_path,
        linked_libraries,
        corelib_path,
        runner_config,
    )?;

    pretty_printing::print_collected_tests_count(
        tests.iter().map(|tests| tests.test_cases.len()).sum(),
        tests.len(),
        package_name,
    );

    let mut tests_iterator = tests.into_iter();

    let mut fuzzing_happened = false;
    let mut summaries = vec![];

    for tests_from_file in tests_iterator.by_ref() {
        let (summary, was_fuzzed) = run_tests_from_file(
            tests_from_file,
            runner_config,
            contracts,
            predeployed_contracts,
            fuzzer_seed,
            cancellation_token.clone(),
        )
        .await?;

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

        let file_summary = TestFileSummary {
            test_case_summaries: skipped,
            runner_exit_status: RunnerStatus::DidNotRun,
            relative_path: tests_from_file.relative_path,
        };
        summaries.push(file_summary);
    }

    pretty_printing::print_test_summary(&summaries);
    if fuzzing_happened {
        pretty_printing::print_test_seed(fuzzer_seed);
    }

    Ok(summaries)
}
async fn run_p_test(
    case: Arc<TestCase>,
    runner: Arc<SierraCasmRunner>,
    contracts_arc: Arc<HashMap<String, StarknetContractArtifacts>>,
    predeployed_contracts_arc: Arc<Utf8PathBuf>,
    fuzzer_runs: u32,
    args: Vec<ConcreteTypeId>,
    fuzzer_seed: u64,
    cancellation_token: Arc<CancellationToken>,
) -> Result<(TestCaseSummary, Option<u32>)> {
    if args.is_empty() {
        match run_from_test_case(
            &runner,
            &case,
            &contracts_arc,
            &predeployed_contracts_arc,
            vec![],
        )
        .await
        {
            Ok(result) => Ok((result, None)),
            Err(e) => Err(e),
        }

        // pretty_printing::print_test_result(&result, None);
        // result
    } else {
        let result = run_with_fuzzing(
            fuzzer_runs,
            &contracts_arc,
            &predeployed_contracts_arc,
            runner,
            case,
            &args,
            fuzzer_seed,
            cancellation_token.clone(),
        )
        .await;

        result
    }
}

async fn run_tests_from_file(
    tests: TestsFromFile,
    runner_config: &RunnerConfig,
    contracts: &HashMap<String, StarknetContractArtifacts>,
    predeployed_contracts: &Utf8PathBuf,
    fuzzer_seed: u64,
    cancellation_token: Arc<CancellationToken>,
) -> Result<(TestFileSummary, bool)> {
    let runner = Arc::new(
        SierraCasmRunner::new(
            tests.sierra_program,
            Some(MetadataComputationConfig::default()),
            OrderedHashMap::default(),
        )
        .context("Failed setting up runner.")?,
    );

    pretty_printing::print_running_tests(&tests.relative_path, tests.test_cases.len());

    let mut was_fuzzed = false;
    let mut tasks = vec![];
    let test_cases = Arc::new(tests.test_cases);
    let contracts_arc = Arc::new(contracts.clone());
    let predeployed_contracts_arc = Arc::new(predeployed_contracts.clone());

    test_cases.iter().for_each(|case| {
        let case_name = case.name.as_str();

        let function = runner.find_function(case_name).unwrap();
        let args = function_args(function, &BUILTINS);

        let task = task::spawn({
            let case = Arc::new(case.clone());
            let c = case.clone();
            let runner = runner.clone();
            let contracts_arc = contracts_arc.clone();
            let predeployed_contracts_arc = predeployed_contracts_arc.clone();
            let fuzzer_runs = runner_config.fuzzer_runs.clone();
            let args: Vec<ConcreteTypeId> = args.into_iter().map(|e| e.clone()).collect();
            let cancellation_token = cancellation_token.clone();
            async move {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        // The token was cancelled
                        dbg!(&c);
                       Ok((TestCaseSummary::skipped(&c), None))
                    },
                    result = run_p_test(case, runner,contracts_arc,predeployed_contracts_arc, fuzzer_runs,args,fuzzer_seed, cancellation_token.clone()) => {
                        result
                    },
                }
            }
        });

        tasks.push(task);

        was_fuzzed = true;
    });

    let mut results = vec![];

    for thread in tasks {
        let (result, runs) = thread.await.unwrap()?;
        if runner_config.exit_first {
            if let TestCaseSummary::Failed { .. } = &result {
                cancellation_token.cancel();
                pretty_printing::print_test_result(&result, None);

                return Ok((
                    TestFileSummary {
                        test_case_summaries: vec![result],
                        runner_exit_status: RunnerStatus::TestFailed,
                        relative_path: tests.relative_path,
                    },
                    was_fuzzed,
                ));
            }
        }
        pretty_printing::print_test_result(&result, runs);
        results.push((result, runs));
    }

    Ok((
        TestFileSummary {
            test_case_summaries: results
                .iter()
                .map(|(result, _runs)| result.clone())
                .collect(),
            runner_exit_status: RunnerStatus::Default,
            relative_path: tests.relative_path,
        },
        was_fuzzed,
    ))
}

async fn run_with_fuzzing(
    fuzzer_runs: u32,
    contracts: &HashMap<String, StarknetContractArtifacts>,
    predeployed_contracts: &Utf8PathBuf,
    runner: Arc<SierraCasmRunner>,
    case: Arc<TestCase>,
    args: &Vec<ConcreteTypeId>,
    fuzzer_seed: u64,
    cancellation_token: Arc<CancellationToken>,
) -> Result<(TestCaseSummary, Option<u32>)> {
    let token = CancellationToken::new();

    if contains_non_felt252_args(args) {
        bail!(
            "Fuzzer only supports felt252 arguments, and test {} defines arguments that are not felt252 type",
            &case.name.as_str()
        );
    }

    let mut fuzzer = RandomFuzzer::new(
        fuzzer_seed,
        fuzzer_runs,
        args.len(),
        &BigUint::zero(),
        &Felt252::prime(),
    );

    let mut tasks = vec![];

    (1..fuzzer_runs).into_iter().for_each(|_| {
        let args = fuzzer.next_felt252_args();
        tasks.push(task::spawn({
            let case = case.clone();
            let runner = runner.clone();
            let contracts_arc = Arc::new(contracts.clone());
            let predeployed_contracts_arc = Arc::new(predeployed_contracts.clone());
            let args = args.clone();
            let cancellation_token = cancellation_token.clone();
            let cancellation_fuzzing_token = Arc::new(token.clone());
            async move {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        // The token was cancelled
                        Ok(TestCaseSummary::Failed { name: "fuzzing".to_string(), run_result: None, msg: None, arguments: vec![] })
                    },
                    _ = cancellation_fuzzing_token.cancelled() => {
                        Ok(TestCaseSummary::Failed { name: "fuzzing".to_string(), run_result: None, msg: None, arguments: vec![] })
                    },
                    result = run_from_test_case(
                        &runner,
                        &case,
                        &contracts_arc,
                        &predeployed_contracts_arc,
                        args.clone(),
                    ) => {
                        result
                    },
                }
            }
        }))
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

fn contains_non_felt252_args(args: &Vec<ConcreteTypeId>) -> bool {
    args.iter().any(|pt| {
        if let Some(name) = &pt.debug_name {
            return name != &SmolStr::from("felt252");
        }
        false
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
    use assert_fs::fixture::PathCopy;
    use test_collector::ExpectedTestResult;

    #[test]
    fn runner_config_default_arguments() {
        let config = RunnerConfig::new(None, false, false, None, None, &Default::default());
        assert_eq!(
            config,
            RunnerConfig {
                test_name_filter: None,
                exact_match: false,
                exit_first: false,
                fuzzer_runs: FUZZER_RUNS_DEFAULT,
                fuzzer_seed: None,
            }
        );
    }

    #[test]
    fn runner_config_just_scarb_arguments() {
        let config_from_scarb = ForgeConfig {
            exit_first: true,
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
                fuzzer_runs: 1234,
                fuzzer_seed: Some(500),
            }
        );
    }

    #[test]
    fn runner_config_argument_precedence() {
        let config_from_scarb = ForgeConfig {
            exit_first: false,
            fuzzer_runs: Some(1234),
            fuzzer_seed: Some(1000),
        };
        let config = RunnerConfig::new(None, false, true, Some(100), Some(32), &config_from_scarb);
        assert_eq!(
            config,
            RunnerConfig {
                test_name_filter: None,
                exact_match: false,
                exit_first: true,
                fuzzer_runs: 100,
                fuzzer_seed: Some(32),
            }
        );
    }

    #[test]
    fn collecting_tests() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let tests_path = Utf8PathBuf::from_path_buf(temp.to_path_buf()).unwrap();
        let lib_path = tests_path.join("src/lib.cairo");

        let tests = find_test_files(&tests_path, &lib_path).unwrap();

        assert!(!tests.is_empty());
    }

    #[test]
    fn filtering_tests() {
        let mocked_tests: Vec<TestCase> = vec![
            TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
            },
            TestCase {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
            },
            TestCase {
                name: "outer::crate2::execute_next_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
            },
        ];

        let filtered = filter_tests_by_name("do", false, mocked_tests.clone());
        assert_eq!(
            filtered,
            vec![TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
            },]
        );

        let filtered = filter_tests_by_name("run", false, mocked_tests.clone());
        assert_eq!(
            filtered,
            vec![TestCase {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
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
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                },
                TestCase {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
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
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                },
                TestCase {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
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
            },
            TestCase {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
            },
            TestCase {
                name: "outer::crate2::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
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
                },
                TestCase {
                    name: "outer::crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
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
            },
            TestCase {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
            },
            TestCase {
                name: "outer::crate3::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
            },
            TestCase {
                name: "do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
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
            },]
        );

        let filtered = filter_tests_by_name("crate1::do_thing", true, mocked_tests.clone());
        assert_eq!(
            filtered,
            vec![TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
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
            },
            TestCase {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
            },
            TestCase {
                name: "thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
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
                },
                TestCase {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                },
                TestCase {
                    name: "thing".to_string(),
                    available_gas: None,
                    expected_result: ExpectedTestResult::Success,
                },
            ]
        );
    }

    #[test]
    fn args_with_only_felt252() {
        let typ = ConcreteTypeId {
            id: 0,
            debug_name: Some(SmolStr::from("felt252")),
        };
        let args = vec![&typ, &typ];
        assert!(!contains_non_felt252_args(&args));
    }

    #[test]
    fn args_with_not_felt252() {
        let typ = ConcreteTypeId {
            id: 0,
            debug_name: Some(SmolStr::from("felt252")),
        };
        let typ2 = ConcreteTypeId {
            id: 0,
            debug_name: Some(SmolStr::from("Uint256")),
        };
        let args = vec![&typ, &typ, &typ2];
        assert!(contains_non_felt252_args(&args));
    }
}
