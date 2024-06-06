use super::with_config;
use crate::{
    block_number_map::BlockNumberMap,
    combine_configs::combine_configs,
    pretty_printing,
    scarb::{
        config::{ForgeConfigFromScarb, ForkTarget},
        load_test_artifacts,
    },
    shared_cache::FailedTestsCache,
    test_filter::TestsFilter,
    warn::{
        warn_if_available_gas_used_with_incompatible_scarb_version,
        warn_if_incompatible_rpc_version,
    },
    TestArgs,
};
use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use configuration::load_package_config;
use forge_runner::{
    compiled_runnable::{TestTargetWithConfig, TestTargetWithResolvedConfig},
    forge_config::ForgeConfig,
    test_case_summary::AnyTestCaseSummary,
    test_crate_summary::TestCrateSummary,
    TestCrateRunResult,
};
use scarb_api::get_contracts_artifacts_and_source_sierra_paths;
use scarb_metadata::{Metadata, PackageMetadata};
use std::{env, sync::Arc};

pub struct RunFromCrateArgs {
    pub compiled_test_crates: Vec<TestTargetWithConfig>,
    pub tests_filter: TestsFilter,
    pub forge_config: Arc<ForgeConfig>,
    pub fork_targets: Vec<ForkTarget>,
    pub package_name: String,
}

pub fn prepare_crate(
    package: PackageMetadata,
    scarb_metadata: &Metadata,
    args: &TestArgs,
    cache_dir: Utf8PathBuf,
    snforge_target_dir_path: &Utf8Path,
    versioned_programs_dir: Utf8PathBuf,
) -> Result<RunFromCrateArgs> {
    env::set_current_dir(&package.root)?;

    let compiled_test_crates = load_test_artifacts(snforge_target_dir_path, &package.name)?;

    let contracts =
        get_contracts_artifacts_and_source_sierra_paths(scarb_metadata, &package.id, None)?;
    let contracts_data = ContractsData::try_from(contracts)?;

    let forge_config_from_scarb =
        load_package_config::<ForgeConfigFromScarb>(scarb_metadata, &package.id)?;
    let forge_config = Arc::new(combine_configs(
        args.exit_first,
        args.fuzzer_runs,
        args.fuzzer_seed,
        args.detailed_resources,
        args.save_trace_data,
        args.build_profile,
        args.max_n_steps,
        contracts_data,
        cache_dir.clone(),
        versioned_programs_dir,
        &forge_config_from_scarb,
    ));

    let test_filter = TestsFilter::from_flags(
        args.test_filter.clone(),
        args.exact,
        args.only_ignored,
        args.include_ignored,
        args.rerun_failed,
        FailedTestsCache::new(cache_dir),
    );

    Ok(RunFromCrateArgs {
        compiled_test_crates: compiled_test_crates.into_iter().map(From::from).collect(),
        forge_config,
        tests_filter: test_filter,
        fork_targets: forge_config_from_scarb.fork,
        package_name: package.name,
    })
}

pub async fn run_from_crate(
    RunFromCrateArgs {
        compiled_test_crates,
        forge_config,
        tests_filter,
        fork_targets,
        package_name,
    }: RunFromCrateArgs,
    block_number_map: &mut BlockNumberMap,
) -> Result<Vec<TestCrateSummary>> {
    let mut test_targets_with_resolved_config: Vec<TestTargetWithResolvedConfig> =
        Vec::with_capacity(compiled_test_crates.len());

    for compiled_test_crate in compiled_test_crates {
        let compiled_test_crate =
            with_config(compiled_test_crate, &fork_targets, block_number_map).await?;

        test_targets_with_resolved_config.push(compiled_test_crate);
    }

    let all_tests: usize = test_targets_with_resolved_config
        .iter()
        .map(|tc| tc.test_cases.len())
        .sum();

    let test_crates = test_targets_with_resolved_config
        .into_iter()
        .map(|mut tc| {
            tests_filter.filter_tests(&mut tc.test_cases)?;
            Ok(tc)
        })
        .collect::<Result<Vec<TestTargetWithResolvedConfig>>>()?;
    let not_filtered: usize = test_crates.iter().map(|tc| tc.test_cases.len()).sum();
    let filtered = all_tests - not_filtered;

    warn_if_available_gas_used_with_incompatible_scarb_version(&test_crates)?;
    warn_if_incompatible_rpc_version(&test_crates).await?;

    pretty_printing::print_collected_tests_count(not_filtered, &package_name);

    let mut summaries = vec![];

    for compiled_test_crate in test_crates {
        pretty_printing::print_running_tests(
            compiled_test_crate.tests_location,
            compiled_test_crate.test_cases.len(),
        );

        let forge_config = forge_config.clone();

        let summary = forge_runner::run_tests_from_crate(
            compiled_test_crate,
            forge_config,
            &tests_filter,
            &package_name,
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
