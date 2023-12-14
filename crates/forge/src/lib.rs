use anyhow::{anyhow, Result};
use camino::Utf8Path;

use crate::scarb::load_test_artifacts;
use forge_runner::test_case_summary::AnyTestCaseSummary;
use std::sync::Arc;

use compiled_raw::{CompiledTestCrateRaw, RawForkConfig, RawForkParams};
use forge_runner::test_crate_summary::TestCrateSummary;
use forge_runner::{RunnerConfig, RunnerParams, TestCrateRunResult};

use crate::block_number_map::BlockNumberMap;
use forge_runner::compiled_runnable::{CompiledTestCrateRunnable, TestCaseRunnable};

use crate::scarb::config::ForkTarget;
use crate::test_filter::TestsFilter;

pub mod block_number_map;
pub mod compiled_raw;
pub mod pretty_printing;
pub mod scarb;
pub mod shared_cache;
pub mod test_filter;

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

async fn to_runnable(
    compiled_test_crate: CompiledTestCrateRaw,
    fork_targets: &[ForkTarget],
    block_number_map: &mut BlockNumberMap,
) -> Result<CompiledTestCrateRunnable> {
    let mut test_cases = vec![];

    for case in compiled_test_crate.test_cases {
        let fork_config = if let Some(fc) = case.fork_config {
            let raw_fork_params = replace_id_with_params(fc, fork_targets)?;
            let fork_config = block_number_map
                .validated_fork_config_from_fork_params(&raw_fork_params)
                .await?;
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

    Ok(CompiledTestCrateRunnable {
        sierra_program: compiled_test_crate.sierra_program,
        test_cases,
    })
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
pub async fn run(
    package_name: &str,
    snforge_target_dir_path: &Utf8Path,
    tests_filter: &TestsFilter,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    fork_targets: &[ForkTarget],
    block_number_map: &mut BlockNumberMap,
) -> Result<Vec<TestCrateSummary>> {
    let test_crates = load_test_artifacts(snforge_target_dir_path, package_name)?;
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

        let compiled_test_crate =
            to_runnable(compiled_test_crate, fork_targets, block_number_map).await?;
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

    let any_fuzz_test_was_run = summaries.iter().any(|crate_summary| {
        crate_summary
            .test_case_summaries
            .iter()
            .filter(|summary| matches!(summary, AnyTestCaseSummary::Fuzzing(_)))
            .any(|summary| summary.is_passed() || summary.is_failed())
    });

    if any_fuzz_test_was_run {
        pretty_printing::print_test_seed(runner_config.fuzzer_seed);
    }

    Ok(summaries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiled_raw::{CompiledTestCrateRaw, CrateLocation, TestCaseRaw};
    use cairo_lang_sierra::program::Program;
    use forge_runner::expected_result::ExpectedTestResult;

    #[tokio::test]
    async fn to_runnable_unparsable_url() {
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

        assert!(
            to_runnable(mocked_tests, &[], &mut BlockNumberMap::default())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn to_runnable_non_existent_id() {
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
            &mut BlockNumberMap::default()
        )
        .await
        .is_err());
    }
}
