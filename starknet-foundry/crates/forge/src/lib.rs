use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use test_results::TestResult;
use walkdir::WalkDir;

use cairo_lang_runner::SierraCasmRunner;
use cairo_lang_runner::{RunResultValue, SierraCasmRunner};
use cairo_lang_sierra::program::Program;
use cairo_lang_sierra_to_casm::metadata::MetadataComputationConfig;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;

use crate::running::run_from_test_units;
use crate::scarb::StarknetContractArtifacts;
use test_collector::{collect_tests, LinkedLibrary, TestUnit};

use crate::test_results::TestSummary;

// Reexport modules in lib.rs scope
pub use crate::running::TestUnitSummary;

pub mod pretty_printing;
pub mod scarb;

mod cheatcodes_hint_processor;
mod running;
mod test_results;

/// Configuration of the test runner
#[derive(Deserialize, Debug, PartialEq, Default)]
pub struct RunnerConfig {
    test_name_filter: Option<String>,
    exact_match: bool,
    exit_first: bool,
}

impl RunnerConfig {
    #[must_use]
    pub fn new(
        test_name_filter: Option<String>,
        exact_match: bool,
        exit_first: bool,
        protostar_config_from_scarb: &ProtostarConfigFromScarb,
    ) -> Self {
        Self {
            test_name_filter,
            exact_match,
            exit_first: protostar_config_from_scarb.exit_first || exit_first,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RunnerStatus {
    Default,
    TestFailed,
}

/// Represents protostar config deserialized from Scarb.toml
#[derive(Deserialize, Debug, PartialEq, Default)]
pub struct ProtostarConfigFromScarb {
    #[serde(default)]
    exit_first: bool,
}

struct TestsFromFile {
    sierra_program: Program,
    test_units: Vec<TestUnit>,
    relative_path: Utf8PathBuf,
}

fn collect_tests_from_directory(
    input_path: &Utf8PathBuf,
    linked_libraries: Option<Vec<LinkedLibrary>>,
    corelib_path: Option<&Utf8PathBuf>,
    runner_config: &RunnerConfig,
) -> Result<Vec<TestsFromFile>> {
    let test_files = find_cairo_files_in_directory(input_path)?;
    internal_collect_tests(
        input_path,
        linked_libraries,
        test_files,
        corelib_path,
        runner_config,
    )
}

fn find_cairo_files_in_directory(input_path: &Utf8PathBuf) -> Result<Vec<Utf8PathBuf>> {
    let mut test_files: Vec<Utf8PathBuf> = vec![];

    for entry in WalkDir::new(input_path).sort_by(|a, b| a.file_name().cmp(b.file_name())) {
        let entry =
            entry.with_context(|| format!("Failed to read directory at path = {input_path}"))?;
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

fn internal_collect_tests(
    input_path: &Utf8PathBuf,
    linked_libraries: Option<Vec<LinkedLibrary>>,
    test_files: Vec<Utf8PathBuf>,
    corelib_path: Option<&Utf8PathBuf>,
    runner_config: &RunnerConfig,
) -> Result<Vec<TestsFromFile>> {
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

    let linked_libraries = linked_libraries;

    let mut tests = vec![];
    for ref test_file in test_files {
        let (sierra_program, tests_configs) = collect_tests(
            test_file.as_str(),
            None,
            linked_libraries.clone(),
            Some(builtins.clone()),
            corelib_path.map(|corelib_path| corelib_path.as_str()),
        )?;

        let test_units = strip_path_from_test_names(tests_configs)?;
        let test_units = if let Some(test_name_filter) = &runner_config.test_name_filter {
            filter_tests_by_name(test_name_filter, runner_config.exact_match, test_units)?
        } else {
            test_units
        };

        let relative_path = test_file.strip_prefix(input_path)?.to_path_buf();
        tests.push(TestsFromFile {
            sierra_program,
            test_units,
            relative_path,
        });
    }

    Ok(tests)
}

// TODO consider resolving this
#[allow(clippy::implicit_hasher)]
pub fn run(
    input_path: &Utf8PathBuf,
    linked_libraries: Option<Vec<LinkedLibrary>>,
    runner_config: &RunnerConfig,
    corelib_path: Option<&Utf8PathBuf>,
    contracts: &HashMap<String, StarknetContractArtifacts>,
) -> Result<Vec<TestFileSummary>> {
    let tests =
        collect_tests_from_directory(input_path, linked_libraries, corelib_path, runner_config)?;

    pretty_printing::print_collected_tests_count(
        tests.iter().map(|tests| tests.test_units.len()).sum(),
        tests.len(),
    );

    let mut tests_summary = TestSummary::default();
    let mut tests_iterator = tests.into_iter();

    let mut summaries = vec![];
    for tests_from_file in tests_iterator.by_ref() {
        let summary =
            run_tests_from_file(tests_from_file, &mut tests_summary, runner_config, contracts)?
           ;
        summaries.push(summary.clone());
        if summary.runner_exit_status == RunnerStatus::TestFailed {
            break;
        }
    }

    for tests_from_file in tests_iterator {
        for unit in tests_from_file.test_units {
            let skipped_result = skip_from_test_config(&unit);
            pretty_printing::print_test_result(&skipped_result);
            tests_summary.update(skipped_result);
        }
    }

    pretty_printing::print_test_summary(&tests_summary);
    Ok(summaries)
}

#[derive(Debug, PartialEq, Clone)]
pub struct TestFileSummary {
    pub test_unit_summaries: Vec<TestUnitSummary>,
    pub runner_exit_status: RunnerStatus,
    pub relative_path: Utf8PathBuf,
}

fn run_tests_from_file(
    tests: TestsFromFile,
    tests_summary: &mut TestSummary,
    runner_config: &RunnerConfig,
    contracts: &HashMap<String, StarknetContractArtifacts>,
) -> Result<TestFileSummary> {
    let mut runner = SierraCasmRunner::new(
        tests.sierra_program,
        Some(MetadataComputationConfig::default()),
        OrderedHashMap::default(),
    )
    .context("Failed setting up runner.")?;

    pretty_printing::print_running_tests(&tests.relative_path, tests.test_units.len());
    let mut results = vec![];
    for (i, unit) in tests.test_units.iter().enumerate() {
        let result = run_from_test_units(&mut runner, unit, contracts)?;
        results.push(result.clone());

        pretty_printing::print_test_result(&result);
        if runner_config.exit_first {
            if let TestResult::Failed { .. } = result {
                for config in &tests.tests_configs[i + 1..] {
                    let skipped_result = skip_from_test_config(config);
                    pretty_printing::print_test_result(&skipped_result);
                    tests_summary.update(skipped_result);
                }
                tests_summary.update(result);
                return Ok(RunTestsSummary {
                    test_run_summaries: results,
                    runner_exit_status: RunnerStatus::TestFailed,
                    relative_path: tests.relative_path,
                });
            }
        }

        tests_summary.update(result.value);
    }
    Ok(TestFileSummary {
        test_unit_summaries: results,
        runner_exit_status: RunnerStatus::Default,
        relative_path: tests.relative_path,
    })
}

fn strip_path_from_test_names(test_units: Vec<TestUnit>) -> Result<Vec<TestUnit>> {
    test_units
        .into_iter()
        .map(|test_unit| {
            let name: String = test_unit
                .name
                .rsplit('/')
                .next()
                .with_context(|| format!("Failed to get test name from = {}", test_unit.name))?
                .into();

            Ok(TestUnit {
                name,
                available_gas: test_unit.available_gas,
            })
        })
        .collect()
}

fn filter_tests_by_name(
    test_name_filter: &str,
    exact_match: bool,
    test_units: Vec<TestUnit>,
) -> Result<Vec<TestUnit>> {
    let mut result = vec![];
    for test in test_units {
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

fn test_name_contains(test_name_filter: &str, test: &TestUnit) -> Result<bool> {
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

    #[test]
    fn collecting_tests() {
        let temp = assert_fs::TempDir::new().unwrap();
        temp.copy_from("tests/data/simple_test", &["**/*.cairo", "**/*.toml"])
            .unwrap();
        let tests_path = Utf8PathBuf::from_path_buf(temp.to_path_buf()).unwrap();

        let tests = find_cairo_files_in_directory(&tests_path).unwrap();

        assert!(!tests.is_empty());
    }

    #[test]
    fn collecting_tests_err_on_invalid_dir() {
        let tests_path = Utf8PathBuf::from("aaee");

        let result = find_cairo_files_in_directory(&tests_path);
        let err = result.unwrap_err();

        assert!(err.to_string().contains("Failed to read directory at path"));
    }

    #[test]
    fn filtering_tests() {
        let mocked_tests: Vec<TestUnit> = vec![
            TestUnit {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
            },
            TestUnit {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
            },
            TestUnit {
                name: "outer::crate2::execute_next_thing".to_string(),
                available_gas: None,
            },
        ];

        let filtered = filter_tests_by_name("do", false, mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered,
            vec![TestUnit {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
            },]
        );

        let filtered = filter_tests_by_name("run", false, mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered,
            vec![TestUnit {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
            },]
        );

        let filtered = filter_tests_by_name("thing", false, mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered,
            vec![
                TestUnit {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                },
                TestUnit {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                },
                TestUnit {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                },
            ]
        );

        let filtered = filter_tests_by_name("nonexistent", false, mocked_tests.clone()).unwrap();
        assert_eq!(filtered, vec![]);

        let filtered = filter_tests_by_name("", false, mocked_tests).unwrap();
        assert_eq!(
            filtered,
            vec![
                TestUnit {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                },
                TestUnit {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                },
                TestUnit {
                    name: "outer::crate2::execute_next_thing".to_string(),
                    available_gas: None,
                },
            ]
        );
    }

    #[test]
    fn filtering_tests_only_uses_name() {
        let mocked_tests: Vec<TestUnit> = vec![
            TestUnit {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
            },
            TestUnit {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
            },
            TestUnit {
                name: "outer::crate2::run_other_thing".to_string(),
                available_gas: None,
            },
        ];

        let filtered = filter_tests_by_name("crate", false, mocked_tests).unwrap();
        assert_eq!(filtered, vec![]);
    }

    #[test]
    fn filtering_with_exact_match() {
        let mocked_tests: Vec<TestUnit> = vec![
            TestUnit {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
            },
            TestUnit {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
            },
            TestUnit {
                name: "outer::crate3::run_other_thing".to_string(),
                available_gas: None,
            },
            TestUnit {
                name: "do_thing".to_string(),
                available_gas: None,
            },
        ];

        let filtered = filter_tests_by_name("", true, mocked_tests.clone()).unwrap();
        assert_eq!(filtered, vec![]);

        let filtered = filter_tests_by_name("thing", true, mocked_tests.clone()).unwrap();
        assert_eq!(filtered, vec![]);

        let filtered = filter_tests_by_name("do_thing", true, mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered,
            vec![TestUnit {
                name: "do_thing".to_string(),
                available_gas: None,
            },]
        );

        let filtered =
            filter_tests_by_name("crate1::do_thing", true, mocked_tests.clone()).unwrap();
        assert_eq!(
            filtered,
            vec![TestUnit {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
            },]
        );

        let filtered =
            filter_tests_by_name("crate3::run_other_thing", true, mocked_tests.clone()).unwrap();
        assert_eq!(filtered, vec![]);

        let filtered =
            filter_tests_by_name("outer::crate3::run_other_thing", true, mocked_tests).unwrap();
        assert_eq!(
            filtered,
            vec![TestUnit {
                name: "outer::crate3::run_other_thing".to_string(),
                available_gas: None,
            },]
        );
    }

    #[test]
    fn filtering_tests_works_without_crate_in_test_name() {
        let mocked_tests: Vec<TestUnit> = vec![
            TestUnit {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
            },
            TestUnit {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
            },
            TestUnit {
                name: "thing".to_string(),
                available_gas: None,
            },
        ];

        let result = filter_tests_by_name("thing", false, mocked_tests).unwrap();
        assert_eq!(
            result,
            vec![
                TestUnit {
                    name: "crate1::do_thing".to_string(),
                    available_gas: None,
                },
                TestUnit {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                },
                TestUnit {
                    name: "thing".to_string(),
                    available_gas: None,
                },
            ]
        );
    }

    #[test]
    fn strip_path() {
        let mocked_tests: Vec<TestUnit> = vec![
            TestUnit {
                name: "/Users/user/protostar/protostar-rust/tests/data/simple_test/src::test::test_fib".to_string(),
                available_gas: None,
            },
            TestUnit {
                name: "crate2::run_other_thing".to_string(),
                available_gas: None,
            },
            TestUnit {
                name: "src/crate2::run_other_thing".to_string(),
                available_gas: None,
            },
        ];

        let striped_tests = strip_path_from_test_names(mocked_tests).unwrap();
        assert_eq!(
            striped_tests,
            vec![
                TestUnit {
                    name: "src::test::test_fib".to_string(),
                    available_gas: None,
                },
                TestUnit {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                },
                TestUnit {
                    name: "crate2::run_other_thing".to_string(),
                    available_gas: None,
                },
            ]
        );
    }
}
