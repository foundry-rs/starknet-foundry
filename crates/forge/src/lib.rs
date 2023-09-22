use std::collections::HashMap;
use std::fmt::Debug;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context, Result};
use ark_std::iterable::Iterable;
use assert_fs::fixture::{FileTouch, PathChild, PathCopy};
use assert_fs::TempDir;
use cairo_felt::Felt252;
use camino::Utf8PathBuf;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use test_case_summary::TestCaseSummary;

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
pub use crate::test_crate_summary::TestCrateSummary;
use test_collector::{collect_tests, LinkedLibrary, TestCase, TEST_PACKAGE_NAME};

pub mod pretty_printing;
pub mod scarb;
pub mod test_case_summary;

mod cheatcodes_hint_processor;
mod fuzzer;
mod running;
mod test_crate_summary;

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

fn collect_tests_from_package(
    package_path: &Utf8PathBuf,
    package_name: &str,
    lib_path: &Utf8PathBuf,
    mut linked_libraries: Vec<LinkedLibrary>,
    corelib_path: &Utf8PathBuf,
    runner_config: &RunnerConfig,
) -> Result<Vec<TestsFromCrate>> {
    let maybe_tests_tmp_dir = pack_tests_into_one_file(package_path)?;

    let mut all_test_roots = vec![(lib_path.clone(), package_name, TestCrateType::Lib)];

    if let Some(tests_tmp_dir) = &maybe_tests_tmp_dir {
        let tests_tmp_dir_path = Utf8PathBuf::from_path_buf(tests_tmp_dir.to_path_buf().clone())
            .map_err(|_| anyhow!("Failed to convert tests temporary directory to Utf8PathBuf"))?;
        let tests_lib_path = tests_tmp_dir_path.join("lib.cairo");

        all_test_roots.push((tests_lib_path, &TEST_PACKAGE_NAME, TestCrateType::Tests));

        linked_libraries.push(LinkedLibrary {
            name: TEST_PACKAGE_NAME.to_string(),
            path: PathBuf::from(tests_tmp_dir_path),
        });
    }

    let tests_from_files = all_test_roots
        .par_iter()
        .map(|(test_root, crate_name, crate_type)| {
            collect_tests_from_tree(
                test_root,
                crate_name,
                *crate_type,
                &linked_libraries,
                corelib_path,
                runner_config,
            )
        })
        .collect();

    if let Some(tests_tmp_dir) = maybe_tests_tmp_dir {
        let path = tests_tmp_dir.path().to_path_buf();
        tests_tmp_dir.close().with_context(|| {
            anyhow!(
            "Failed to close temporary directory = {} with test files. The files might have not been released from filesystem",
            path.display()
        )
        })?;
    }

    tests_from_files
}

fn pack_tests_into_one_file(package_path: &Utf8PathBuf) -> Result<Option<TempDir>> {
    let tests_folder_path = package_path.join("tests");
    if !tests_folder_path.try_exists()? {
        return Ok(None);
    }

    let tmp_dir = TempDir::new()?;
    tmp_dir
        .copy_from(&tests_folder_path, &["**/*.cairo"])
        .context("Unable to copy files to temporary directory")?;

    let tests_lib_path = tmp_dir.child("lib.cairo");
    if tests_lib_path.try_exists()? {
        return Ok(Some(tmp_dir));
    }
    tests_lib_path.touch()?;

    let mut content = String::new();
    for entry in std::fs::read_dir(&tests_folder_path)? {
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
    Ok(Some(tmp_dir))
}

fn collect_tests_from_tree(
    test_root: &Utf8PathBuf,
    crate_name: &str,
    crate_type: TestCrateType,
    linked_libraries: &Vec<LinkedLibrary>,
    corelib_path: &Utf8PathBuf,
    runner_config: &RunnerConfig,
) -> Result<TestsFromCrate> {
    let (sierra_program, tests_configs) = collect_tests(
        test_root.as_str(),
        None,
        crate_name,
        linked_libraries,
        Some(BUILTINS.clone()),
        corelib_path.into(),
    )?;

    let test_cases = if let Some(test_name_filter) = &runner_config.test_name_filter {
        filter_tests_by_name(test_name_filter, runner_config.exact_match, tests_configs)
    } else {
        tests_configs
    };

    Ok(TestsFromCrate {
        sierra_program,
        test_cases,
        test_crate_type: crate_type,
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
    linked_libraries: Vec<LinkedLibrary>,
    runner_config: &RunnerConfig,
    corelib_path: &Utf8PathBuf,
    contracts: &HashMap<String, StarknetContractArtifacts>,
    predeployed_contracts: &Utf8PathBuf,
    fuzzer_seed: u64,
) -> Result<Vec<TestCrateSummary>> {
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

    for tests_from_crate in tests_iterator.by_ref() {
        let (summary, was_fuzzed) = run_tests_from_crate(
            tests_from_crate,
            runner_config,
            contracts,
            predeployed_contracts,
            fuzzer_seed,
        )?;

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
        pretty_printing::print_test_seed(fuzzer_seed);
    }

    Ok(summaries)
}

fn run_tests_from_crate(
    tests: TestsFromCrate,
    runner_config: &RunnerConfig,
    contracts: &HashMap<String, StarknetContractArtifacts>,
    predeployed_contracts: &Utf8PathBuf,
    fuzzer_seed: u64,
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
            let result =
                run_from_test_case(&runner, case, contracts, predeployed_contracts, vec![])?;
            pretty_printing::print_test_result(&result, None);

            result
        } else {
            was_fuzzed = true;
            let (result, runs) = run_with_fuzzing(
                runner_config,
                contracts,
                predeployed_contracts,
                &runner,
                case,
                &args,
                fuzzer_seed,
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
    runner_config: &RunnerConfig,
    contracts: &HashMap<String, StarknetContractArtifacts>,
    predeployed_contracts: &Utf8PathBuf,
    runner: &SierraCasmRunner,
    case: &TestCase,
    args: &Vec<&ConcreteTypeId>,
    fuzzer_seed: u64,
) -> Result<(TestCaseSummary, u32)> {
    if contains_non_felt252_args(args) {
        bail!(
            "Fuzzer only supports felt252 arguments, and test {} defines arguments that are not felt252 type",
            case.name.as_str()
        );
    }

    let mut fuzzer = RandomFuzzer::new(
        fuzzer_seed,
        runner_config.fuzzer_runs,
        args.len(),
        &BigUint::zero(),
        &Felt252::prime(),
    );

    let mut results = vec![];

    for _ in 1..=runner_config.fuzzer_runs {
        let args = fuzzer.next_felt252_args();

        let result =
            run_from_test_case(runner, case, contracts, predeployed_contracts, args.clone())?;
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

fn contains_non_felt252_args(args: &Vec<&ConcreteTypeId>) -> bool {
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
