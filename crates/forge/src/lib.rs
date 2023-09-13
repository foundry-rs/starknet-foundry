use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::{bail, Context, Result};
use ark_std::iterable::Iterable;
use cairo_felt::Felt252;
use camino::{Utf8Path, Utf8PathBuf};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use test_case_summary::TestCaseSummary;
use walkdir::WalkDir;

use cairo_lang_runner::SierraCasmRunner;
use cairo_lang_sierra::ids::ConcreteTypeId;
use cairo_lang_sierra::program::{Function, Program};
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use smol_str::SmolStr;

use crate::fuzzer::{Fuzzer, Random};
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
                .unwrap_or(256),
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

fn collect_tests_from_directory(
    package_path: &Utf8PathBuf,
    package_name: &str,
    lib_path: &Utf8PathBuf,
    linked_libraries: &Option<Vec<LinkedLibrary>>,
    corelib_path: &Utf8PathBuf,
    runner_config: &RunnerConfig,
) -> Result<Vec<TestsFromFile>> {
    let test_files = find_cairo_root_files_in_directory(package_path, lib_path)?;
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

fn find_cairo_root_files_in_directory(
    package_path: &Utf8PathBuf,
    lib_path: &Utf8PathBuf,
) -> Result<Vec<Utf8PathBuf>> {
    let mut test_files: Vec<Utf8PathBuf> = vec![lib_path.clone()];
    let src_path = lib_path.parent().with_context(|| {
        format!("Failed to get parent directory of a package at path = {lib_path}")
    })?;

    for entry in WalkDir::new(package_path)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| !e.path().starts_with(src_path))
    {
        let entry =
            entry.with_context(|| format!("Failed to read directory at path = {package_path}"))?;
        let path = entry.path();

        if path.is_file() && path.extension().unwrap_or_default() == "cairo" {
            test_files.push(
                Utf8Path::from_path(path)
                    .with_context(|| format!("Failed to convert path = {path:?} to utf-8"))?
                    .to_path_buf(),
            );
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
    let builtins = vec![
        "Pedersen",
        "RangeCheck",
        "Bitwise",
        "EcOp",
        "Poseidon",
        "SegmentArena",
        "GasBuiltin",
        "System",
    ];

    let (sierra_program, tests_configs) = collect_tests(
        test_root.as_str(),
        None,
        package_name,
        linked_libraries.clone(),
        Some(builtins.clone()),
        corelib_path.into(),
    )?;

    let test_cases = if let Some(test_name_filter) = &runner_config.test_name_filter {
        filter_tests_by_name(test_name_filter, runner_config.exact_match, tests_configs)?
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
pub fn run(
    package_path: &Utf8PathBuf,
    package_name: &str,
    lib_path: &Utf8PathBuf,
    linked_libraries: &Option<Vec<LinkedLibrary>>,
    runner_config: &RunnerConfig,
    corelib_path: &Utf8PathBuf,
    contracts: &HashMap<String, StarknetContractArtifacts>,
    predeployed_contracts: &Utf8PathBuf,
) -> Result<Vec<TestFileSummary>> {
    let tests = collect_tests_from_directory(
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
    );

    let mut tests_iterator = tests.into_iter();

    let mut fuzzer = match runner_config.fuzzer_seed {
        None => Random::new(),
        Some(seed) => Random::from_seed(seed),
    };

    let mut summaries = vec![];
    for tests_from_file in tests_iterator.by_ref() {
        let summary = run_tests_from_file(
            tests_from_file,
            package_name,
            runner_config,
            contracts,
            predeployed_contracts,
            &mut fuzzer,
        )?;
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
            pretty_printing::print_test_result(test_case_summary);
        }

        let file_summary = TestFileSummary {
            test_case_summaries: skipped,
            runner_exit_status: RunnerStatus::DidNotRun,
            relative_path: tests_from_file.relative_path,
        };
        summaries.push(file_summary);
    }

    pretty_printing::print_test_summary(&summaries);
    Ok(summaries)
}

fn run_tests_from_file(
    tests: TestsFromFile,
    package_name: &str,
    runner_config: &RunnerConfig,
    contracts: &HashMap<String, StarknetContractArtifacts>,
    predeployed_contracts: &Utf8PathBuf,
    fuzzer: &mut dyn Fuzzer,
) -> Result<TestFileSummary> {
    let runner = SierraCasmRunner::new(
        tests.sierra_program,
        Some(MetadataComputationConfig::default()),
        OrderedHashMap::default(),
    )
    .context("Failed setting up runner.")?;

    pretty_printing::print_running_tests(
        &tests.relative_path,
        package_name,
        tests.test_cases.len(),
    );

    let mut results = vec![];
    for (i, case) in tests.test_cases.iter().enumerate() {
        let case_name = case.name.as_str();
        let function = runner.find_function(case_name)?;
        let args = args_for_function(function);

        let builtins = vec![
            "Pedersen",
            "RangeCheck",
            "Bitwise",
            "EcOp",
            "Poseidon",
            "SegmentArena",
            "GasBuiltin",
            "System",
        ];

        let result = if args.is_empty() {
            let result =
                run_from_test_case(&runner, case, contracts, predeployed_contracts, vec![])?;
            pretty_printing::print_test_result(&result);
            result
        } else {
            pretty_printing::print_fuzz_running(case_name, runner_config);

            if contains_non_felt252_args(&args, &builtins) {
                bail!("Test {case_name} requires arguments that are not felt252 type");
            }

            let mut results = vec![];

            for _ in 1..runner_config.fuzzer_runs {
                let args: Vec<Felt252> = args
                    .iter()
                    .flat_map(|_| fuzzer.next_argument("felt252"))
                    .collect();

                let result = run_from_test_case(
                    &runner,
                    case,
                    contracts,
                    predeployed_contracts,
                    args.clone(),
                )?;
                results.push((result.clone(), args));

                if let TestCaseSummary::Failed { .. } = result {
                    // Fuzz failed
                    break;
                }
            }

            let (result, args) = results
                .last()
                .expect("Test should always run at least once")
                .clone();
            let runs = results.len();

            pretty_printing::print_test_result(&result);
            if let TestCaseSummary::Failed { .. } = result {
                pretty_printing::print_fuzz_failed(&args, runs);
            }

            result
        };

        results.push(result.clone());

        if runner_config.exit_first {
            if let TestCaseSummary::Failed { .. } = result {
                for case in &tests.test_cases[i + 1..] {
                    let skipped_result = TestCaseSummary::skipped(case);
                    pretty_printing::print_test_result(&skipped_result);
                    results.push(skipped_result);
                }
                return Ok(TestFileSummary {
                    test_case_summaries: results,
                    runner_exit_status: RunnerStatus::TestFailed,
                    relative_path: tests.relative_path,
                });
            }
        }
    }
    Ok(TestFileSummary {
        test_case_summaries: results,
        runner_exit_status: RunnerStatus::Default,
        relative_path: tests.relative_path,
    })
}

fn contains_non_felt252_args(args: &Vec<&ConcreteTypeId>, builtins: &[&str]) -> bool {
    args.iter().any(|pt| {
        if let Some(name) = &pt.debug_name {
            return name != &SmolStr::from("felt252") && !builtins.contains(&name.as_str());
        }
        false
    })
}

fn args_for_function(function: &Function) -> Vec<&ConcreteTypeId> {
    function
        .signature
        .param_types
        .iter()
        .filter(|pt| pt.debug_name == Some(SmolStr::from("felt252")))
        .collect::<Vec<_>>()
}

fn filter_tests_by_name(
    test_name_filter: &str,
    exact_match: bool,
    test_cases: Vec<TestCase>,
) -> Result<Vec<TestCase>> {
    let mut result = vec![];
    for test in test_cases {
        if exact_match {
            if test.name == test_name_filter {
                result.push(test);
            }
        } else if test_name_contains(test_name_filter, &test)? {
            result.push(test);
        }
    }
    Ok(result)
}

fn test_name_contains(test_name_filter: &str, test: &TestCase) -> Result<bool> {
    let name = test
        .name
        .rsplit("::")
        .next()
        .context(format!("Failed to get test name from = {}", test.name))?;
    Ok(name.contains(test_name_filter))
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
                fuzzer_runs: 256,
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

        let tests = find_cairo_root_files_in_directory(&tests_path, &lib_path).unwrap();

        assert!(!tests.is_empty());
    }

    #[test]
    fn collecting_tests_err_on_invalid_dir() {
        let tests_path = Utf8PathBuf::from("aaee");
        let lib_path = Utf8PathBuf::from("asdfgdfg");

        let result = find_cairo_root_files_in_directory(&tests_path, &lib_path);
        let err = result.unwrap_err();

        assert!(err.to_string().contains("Failed to read directory at path"));
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

        let filtered = filter_tests_by_name("do", false, mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered,
            vec![TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
            },]
        );

        let filtered = filter_tests_by_name("run", false, mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered,
            vec![TestCase {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
            },]
        );

        let filtered = filter_tests_by_name("thing", false, mocked_tests.clone()).unwrap();
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

        let filtered = filter_tests_by_name("nonexistent", false, mocked_tests.clone()).unwrap();
        assert_eq!(filtered, vec![]);

        let filtered = filter_tests_by_name("", false, mocked_tests).unwrap();
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
    fn filtering_tests_only_uses_name() {
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

        let filtered = filter_tests_by_name("crate", false, mocked_tests).unwrap();
        assert_eq!(filtered, vec![]);
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

        let filtered = filter_tests_by_name("", true, mocked_tests.clone()).unwrap();
        assert_eq!(filtered, vec![]);

        let filtered = filter_tests_by_name("thing", true, mocked_tests.clone()).unwrap();
        assert_eq!(filtered, vec![]);

        let filtered = filter_tests_by_name("do_thing", true, mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered,
            vec![TestCase {
                name: "do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
            },]
        );

        let filtered =
            filter_tests_by_name("crate1::do_thing", true, mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered,
            vec![TestCase {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                expected_result: ExpectedTestResult::Success,
            },]
        );

        let filtered =
            filter_tests_by_name("crate3::run_other_thing", true, mocked_tests.clone()).unwrap();
        assert_eq!(filtered, vec![]);

        let filtered =
            filter_tests_by_name("outer::crate3::run_other_thing", true, mocked_tests).unwrap();
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

        let result = filter_tests_by_name("thing", false, mocked_tests).unwrap();
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
        assert!(!contains_non_felt252_args(&args, &[]));
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
        assert!(contains_non_felt252_args(&args, &[]));
    }

    #[test]
    fn args_with_not_felt252_type_builtins() {
        let typ = ConcreteTypeId {
            id: 0,
            debug_name: Some(SmolStr::from("felt252")),
        };
        let typ2 = ConcreteTypeId {
            id: 0,
            debug_name: Some(SmolStr::from("GasBuiltin")),
        };
        let args = vec![&typ, &typ, &typ2];
        assert!(!contains_non_felt252_args(&args, &["GasBuiltin"]));
    }
}
