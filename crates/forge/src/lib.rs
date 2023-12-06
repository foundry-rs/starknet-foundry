use anyhow::{anyhow, Error, Result};
use camino::Utf8Path;
use serde::Deserialize;
use std::fmt::Debug;
use std::sync::Arc;

use forge_runner::test_crate_summary::TestCrateSummary;
use forge_runner::{
    CompiledTestCrateRunnable, RunnerConfig, RunnerParams, TestCaseRunnable, TestCrateRunResult,
    ValidatedForkConfig,
};
use test_collector::{RawForkConfig, RawForkParams};

use crate::collecting::{collect_test_compilation_targets, compile_tests, CompiledTestCrateRaw};
use crate::scarb::config::ForkTarget;
use crate::scarb_test_collector::load_test_artifacts;
use crate::test_filter::TestsFilter;

mod collecting;
pub mod scarb_test_collector;

pub mod pretty_printing;
pub mod scarb;
pub mod shared_cache;
pub mod test_filter;

#[derive(Debug, PartialEq, Clone, Copy, Deserialize)]
pub enum CrateLocation {
    /// Main crate in a package
    Lib,
    /// Crate in the `tests/` directory
    Tests,
}

fn replace_id_with_params(
    raw_fork_config: RawForkConfig,
    fork_targets: &[ForkTarget],
) -> Result<RawForkParams> {
    match raw_fork_config {
        RawForkConfig::Params(raw_fork_params) => Ok(raw_fork_params),
        RawForkConfig::Id(name) => {
            let fork_target_from_runner_config = fork_targets
                .iter()
                .find(|fork| fork.name() == name)
                .ok_or_else(|| {
                    anyhow!("Fork configuration named = {name} not found in the Scarb.toml")
                })?;

            Ok(fork_target_from_runner_config.params().clone())
        }
    }
}

fn to_runnable(
    compiled_test_crate: CompiledTestCrateRaw,
    fork_targets: &[ForkTarget],
) -> Result<CompiledTestCrateRunnable> {
    let mut test_cases = vec![];

    for case in compiled_test_crate.test_cases {
        let fork_config = if let Some(fc) = case.fork_config {
            let raw_fork_params = replace_id_with_params(fc, fork_targets)?;
            let fork_config = ValidatedForkConfig::try_from(raw_fork_params)?;
            Some(fork_config)
        } else {
            None
        };

        test_cases.push(TestCaseRunnable {
            name: case.name,
            available_gas: case.available_gas,
            ignored: case.ignored,
            expected_result: case.expected_result,
            fork_config,
            fuzzer_config: case.fuzzer_config,
        });
    }

    Ok(CompiledTestCrateRunnable::new(
        compiled_test_crate.sierra_program,
        test_cases,
    ))
}

/// Run the tests in the package at the given path
///
/// # Arguments
///
/// * `package_path` - Absolute path to the top-level of the Cairo package
/// * `package_name` - Name of the package specified in Scarb.toml
/// * `package_source_dir_path` - Absolute path to the root of the package
/// * `tests_filter` - `TestFilter` structure used to determine what tests to run
/// * `runner_config` - A configuration of the test runner
/// * `runner_params` - A struct with parameters required to run tests e.g. map with contracts
/// * `fork_target` - A configuration of forks used in tests
#[allow(clippy::implicit_hasher)]
pub async fn run(
    package_path: &Utf8Path,
    package_name: &str,
    package_source_dir_path: &Utf8Path,
    tests_filter: &TestsFilter,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    fork_targets: &[ForkTarget],
) -> Result<Vec<TestCrateSummary>> {
    let compilation_targets =
        collect_test_compilation_targets(package_path, package_name, package_source_dir_path)?;
    let test_crates = compile_tests(&compilation_targets, &runner_params)?;

    run_internal(
        package_name,
        tests_filter,
        runner_config,
        runner_params,
        fork_targets,
        test_crates,
    )
    .await?
}

/// Run the tests in the package at the given path
///
/// # Arguments
///
/// * `package_name` - Name of the package specified in Scarb.toml
/// * `snforge_target_dir_path` - Absolute path to the directory with snforge test artifacts (usually `{package_path}/target/{profile_name}/snforge`)
/// * `tests_filter` - `TestFilter` structure used to determine what tests to run
/// * `runner_config` - A configuration of the test runner
/// * `runner_params` - A struct with parameters required to run tests e.g. map with contracts
/// * `fork_target` - A configuration of forks used in tests
#[allow(clippy::implicit_hasher)]
pub async fn run_with_scarb_collector(
    package_name: &str,
    snforge_target_dir_path: &Utf8Path,
    tests_filter: &TestsFilter,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    fork_targets: &[ForkTarget],
) -> Result<Vec<TestCrateSummary>> {
    let test_crates = load_test_artifacts(snforge_target_dir_path, package_name)?;

    run_internal(
        package_name,
        tests_filter,
        runner_config,
        runner_params,
        fork_targets,
        test_crates,
    )
    .await?
}

async fn run_internal(
    package_name: &str,
    tests_filter: &TestsFilter,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    fork_targets: &[ForkTarget],
    test_crates: Vec<CompiledTestCrateRaw>,
) -> Result<Result<Vec<TestCrateSummary>, Error>, Error> {
    let all_tests: usize = test_crates.iter().map(|tc| tc.test_cases.len()).sum();

    let test_crates = test_crates
        .into_iter()
        .map(|tc| tests_filter.filter_tests(tc))
        .collect::<Result<Vec<CompiledTestCrateRaw>>>()?;
    let not_filtered: usize = test_crates.iter().map(|tc| tc.test_cases.len()).sum();
    let filtered = all_tests - not_filtered;

    pretty_printing::print_collected_tests_count(
        test_crates.iter().map(|tests| tests.test_cases.len()).sum(),
        package_name,
    );

    let mut summaries = vec![];

    for compiled_test_crate in test_crates {
        pretty_printing::print_running_tests(
            compiled_test_crate.tests_location,
            compiled_test_crate.test_cases.len(),
        );

        let compiled_test_crate = to_runnable(compiled_test_crate, fork_targets)?;
        let compiled_test_crate = Arc::new(compiled_test_crate);
        let runner_config = runner_config.clone();
        let runner_params = runner_params.clone();

        let summary = forge_runner::run_tests_from_crate(
            compiled_test_crate.clone(),
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
            _ => unreachable!("Unsupported TestCrateRunResult encountered"),
        }
    }

    pretty_printing::print_test_summary(&summaries, filtered);

    if summaries
        .iter()
        .any(|summary| summary.contained_fuzzed_tests)
    {
        pretty_printing::print_test_seed(runner_config.fuzzer_seed);
    }

    Ok(Ok(summaries))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collecting::CompiledTestCrateRaw;
    use cairo_lang_sierra::program::Program;
    use test_collector::ExpectedTestResult;
    use test_collector::TestCaseRaw;

    #[test]
    fn to_runnable_unparsable_url() {
        let mocked_tests = CompiledTestCrateRaw {
            sierra_program: Program {
                type_declarations: vec![],
                libfunc_declarations: vec![],
                statements: vec![],
                funcs: vec![],
            },
            test_cases: vec![TestCaseRaw {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                ignored: false,
                expected_result: ExpectedTestResult::Success,
                fork_config: Some(RawForkConfig::Params(RawForkParams {
                    url: "unparsable_url".to_string(),
                    block_id_type: "Tag".to_string(),
                    block_id_value: "Latest".to_string(),
                })),
                fuzzer_config: None,
            }],
            tests_location: CrateLocation::Lib,
        };

        assert!(to_runnable(mocked_tests, &[]).is_err());
    }

    #[test]
    fn to_runnable_non_existent_id() {
        let mocked_tests = CompiledTestCrateRaw {
            sierra_program: Program {
                type_declarations: vec![],
                libfunc_declarations: vec![],
                statements: vec![],
                funcs: vec![],
            },
            test_cases: vec![TestCaseRaw {
                name: "crate1::do_thing".to_string(),
                available_gas: None,
                ignored: false,
                expected_result: ExpectedTestResult::Success,
                fork_config: Some(RawForkConfig::Id("non_existent".to_string())),
                fuzzer_config: None,
            }],
            tests_location: CrateLocation::Lib,
        };

        assert!(to_runnable(
            mocked_tests,
            &[ForkTarget::new(
                "definitely_non_existing".to_string(),
                RawForkParams {
                    url: "https://not_taken.com".to_string(),
                    block_id_type: "Number".to_string(),
                    block_id_value: "120".to_string(),
                },
            )],
        )
        .is_err());
    }
}
