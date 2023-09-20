use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::Write;

use anyhow::{anyhow, Context, Result};
use assert_fs::fixture::{FileTouch, PathChild};
use assert_fs::TempDir;
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

fn collect_tests_from_package(
    package_path: &Utf8PathBuf,
    package_name: &str,
    lib_path: &Utf8PathBuf,
    linked_libraries: &Option<Vec<LinkedLibrary>>,
    corelib_path: &Utf8PathBuf,
    runner_config: &RunnerConfig,
) -> Result<Vec<TestsFromFile>> {
    let test_files_from_tests_folder = find_test_files_from_tests_folder(package_path)?;
    let (common_test_file_dir, packed_test_file) =
        pack_tests_into_one_file(&test_files_from_tests_folder, package_path)?;
    let common_test_file_dir_path = Utf8PathBuf::from_path_buf(common_test_file_dir.to_path_buf())
        .map_err(|_| anyhow!("Failed to convert common test file dir to Utf8PathBuf"))?;

    let all_test_roots = vec![
        (lib_path, package_path),
        (&packed_test_file, &common_test_file_dir_path),
    ];
    let tests_from_files = all_test_roots
        .par_iter()
        .map(|(test_file, prefix_path)| {
            collect_tests_from_tree(
                test_file,
                prefix_path,
                package_name,
                linked_libraries,
                corelib_path,
                runner_config,
            )
        })
        .collect();

    let common_test_file_path = common_test_file_dir.path().to_owned().clone();
    common_test_file_dir.close().with_context(|| {
        anyhow!(
            "Failed to close temporary directory = {} with common test file. The file might have not been released from filesystem",
            common_test_file_path.display()
        )
    })?;

    tests_from_files
}

fn find_test_files_from_tests_folder(package_path: &Utf8PathBuf) -> Result<Vec<Utf8PathBuf>> {
    let mut test_files: Vec<Utf8PathBuf> = vec![];
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

fn pack_tests_into_one_file(
    test_files_from_tests_folder: &[Utf8PathBuf],
    package_path: &Utf8PathBuf,
) -> Result<(TempDir, Utf8PathBuf)> {
    let tmp_dir = TempDir::new()?;
    let common_tests_file = tmp_dir.child("tests.cairo");
    common_tests_file.touch()?;
    let mut common_test_file_handle = OpenOptions::new()
        .append(true)
        .open(&common_tests_file)
        .expect("Unable to open common test file");

    for file in test_files_from_tests_folder {
        let content = std::fs::read_to_string(file)?;
        let mod_name = file
            .strip_prefix(package_path.join("tests"))
            .expect("Each test file path should start with package path")
            .to_string()
            .strip_suffix(".cairo")
            .expect("Each test file path should have .cairo extension")
            .replace(['/', '\\'], "_");

        write!(
            &mut common_test_file_handle,
            "\nmod {mod_name} {{\n{content}}}"
        )
        .expect("Failed to write to common test file");
    }

    Ok((
        tmp_dir,
        Utf8PathBuf::from_path_buf(common_tests_file.to_path_buf())
            .map_err(|_| anyhow!("Failed to convert common test file path to Utf8PathBuf"))?,
    ))
}

fn collect_tests_from_tree(
    test_root: &Utf8PathBuf,
    prefix_path: &Utf8PathBuf,
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

    let relative_path = test_root
        .strip_prefix(prefix_path)
        .expect("Test root should be always prefixed with prefix path")
        .to_path_buf();

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

    let mut summaries = vec![];
    for tests_from_file in tests_iterator.by_ref() {
        let summary = run_tests_from_file(
            tests_from_file,
            runner_config,
            contracts,
            predeployed_contracts,
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

    pretty_printing::print_running_tests(&tests.relative_path, tests.test_cases.len());

    let mut results = vec![];
    for (i, case) in tests.test_cases.iter().enumerate() {
        let result = run_from_test_case(&runner, case, contracts, predeployed_contracts)?;
        results.push(result.clone());

        pretty_printing::print_test_result(&result);
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

        let tests = find_test_files_from_tests_folder(&tests_path).unwrap();

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
}
