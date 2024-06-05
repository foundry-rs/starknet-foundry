use crate::{
    block_number_map::BlockNumberMap,
    compiled_raw::{CompiledTestCrateRaw, RawForkConfig, RawForkParams},
    pretty_printing,
    scarb::config::ForkTarget,
    test_filter::TestsFilter,
    warn::{
        warn_if_available_gas_used_with_incompatible_scarb_version,
        warn_if_incompatible_rpc_version,
    },
};
use anyhow::{anyhow, Result};
use forge_runner::{
    forge_config::ForgeConfig, test_case_summary::AnyTestCaseSummary,
    test_crate_summary::TestCrateSummary, TestCrateRunResult,
};
use std::sync::Arc;
use to_runnable::to_runnable;

mod to_runnable;
pub mod workspace;

pub(crate) fn replace_id_with_params<'a>(
    raw_fork_config: &'a RawForkConfig,
    fork_targets: &'a [ForkTarget],
) -> Result<&'a RawForkParams> {
    match raw_fork_config {
        RawForkConfig::Params(raw_fork_params) => Ok(raw_fork_params),
        RawForkConfig::Id(name) => {
            let fork_target_from_runner_config = fork_targets
                .iter()
                .find(|fork| fork.name() == name)
                .ok_or_else(|| {
                    anyhow!("Fork configuration named = {name} not found in the Scarb.toml")
                })?;

            Ok(fork_target_from_runner_config.params())
        }
    }
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
    compiled_test_crates: Vec<CompiledTestCrateRaw>,
    package_name: &str,
    tests_filter: &TestsFilter,
    forge_config: Arc<ForgeConfig>,
    fork_targets: &[ForkTarget],
    block_number_map: &mut BlockNumberMap,
) -> Result<Vec<TestCrateSummary>> {
    let all_tests: usize = compiled_test_crates
        .iter()
        .map(|tc| tc.test_cases.len())
        .sum();

    let test_crates = compiled_test_crates
        .into_iter()
        .map(|mut tc| {
            tests_filter.filter_tests(&mut tc.test_cases)?;
            Ok(tc)
        })
        .collect::<Result<Vec<CompiledTestCrateRaw>>>()?;
    let not_filtered: usize = test_crates.iter().map(|tc| tc.test_cases.len()).sum();
    let filtered = all_tests - not_filtered;

    warn_if_available_gas_used_with_incompatible_scarb_version(&test_crates)?;
    warn_if_incompatible_rpc_version(&test_crates, fork_targets).await?;

    pretty_printing::print_collected_tests_count(
        test_crates.iter().map(|tests| tests.test_cases.len()).sum(),
        package_name,
    );

    let mut summaries = vec![];

    for compiled_test_crate_raw in test_crates {
        pretty_printing::print_running_tests(
            compiled_test_crate_raw.tests_location,
            compiled_test_crate_raw.test_cases.len(),
        );

        let compiled_test_crate =
            to_runnable(compiled_test_crate_raw, fork_targets, block_number_map).await?;
        let forge_config = forge_config.clone();

        let summary = forge_runner::run_tests_from_crate(
            compiled_test_crate,
            forge_config,
            tests_filter,
            package_name,
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
        pretty_printing::print_test_seed(forge_config.test_runner_config.fuzzer_seed);
    }

    Ok(summaries)
}
