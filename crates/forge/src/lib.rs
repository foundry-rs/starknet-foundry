use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::RwLock;
use std::time::Instant;

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use test_case_summary::TestCaseSummary;
use walkdir::WalkDir;

use cairo_lang_runner::SierraCasmRunner;
use cairo_lang_sierra::program::Program;
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;

use crate::running::run_from_test_case;
use crate::scarb::{ForgeConfig, StarknetContractArtifacts};
pub use crate::test_file_summary::TestFileSummary;
use test_collector::{collect_tests, LinkedLibrary, TestCase};

pub mod pretty_printing;
pub mod scarb;
pub mod test_case_summary;

mod cheatcodes_hint_processor;
mod running;
mod test_file_summary;

/// Configuration of the test runner
#[derive(Deserialize, Debug, PartialEq, Default)]
pub struct RunnerConfig {
    test_name_filter: Option<String>,
    exact_match: bool,
    exit_first: bool,
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
        forge_config_from_scarb: &ForgeConfig,
    ) -> Self {
        Self {
            test_name_filter,
            exact_match,
            exit_first: forge_config_from_scarb.exit_first || exit_first,
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

    let start = Instant::now();

    let tests = collect_tests_from_directory(
        package_path,
        package_name,
        lib_path,
        linked_libraries,
        corelib_path,
        runner_config,
    )?;

    let duration = start.elapsed();
    println!("Time elapsed collect_tests_from_directory: {:?}", duration);

    pretty_printing::print_collected_tests_count(
        tests.iter().map(|tests| tests.test_cases.len()).sum(),
        tests.len(),
    );

    let mut tests_iterator = tests.into_iter();

    let start = Instant::now();

    let mut summaries = vec![];
    for tests_from_file in tests_iterator.by_ref() {
        let summary = run_tests_from_file(
            tests_from_file,
            package_name,
            runner_config,
            contracts,
            predeployed_contracts,
        )?;
        summaries.push(summary.clone());
        if summary.runner_exit_status == RunnerStatus::TestFailed {
            break;
        }
    }

    let duration = start.elapsed();
    println!("Time elapsed run_tests_from_file: {:?}", duration);

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

    let skip_tests_lock = RwLock::new(false);
    let results = tests.test_cases.par_iter().map(|case| {
        if runner_config.exit_first {
            if *skip_tests_lock.read().unwrap() {
                let skipped_result = TestCaseSummary::skipped(case);
                pretty_printing::print_test_result(&skipped_result);
                return skipped_result;
            }
        }

        let result = run_from_test_case(&runner, case, contracts, predeployed_contracts).unwrap();

        pretty_printing::print_test_result(&result);

        if runner_config.exit_first {
            if let TestCaseSummary::Failed { .. } = result {
                let mut skip_tests = skip_tests_lock.write().unwrap();
                *skip_tests = true;
            }
        }

        result
    }).collect();

    Ok(TestFileSummary {
        test_case_summaries: results,
        runner_exit_status: RunnerStatus::Default,
        relative_path: tests.relative_path,
    })
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
}
