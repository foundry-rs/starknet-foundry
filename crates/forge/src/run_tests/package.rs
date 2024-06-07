use super::{
    resolve_config::resolve_config,
    test_target::{run_test_target, TestTargetRunResult},
};
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
    forge_config::ForgeConfig,
    package_tests::{
        raw::TestCrateRaw, with_config::TestTargetWithConfig,
        with_config_resolved::TestTargetWithResolvedConfig,
    },
    test_case_summary::AnyTestCaseSummary,
    test_crate_summary::TestTargetSummary,
};
use scarb_api::get_contracts_artifacts_and_source_sierra_paths;
use scarb_metadata::{Metadata, PackageMetadata};
use std::{env, sync::Arc};

pub struct RunFromCrateArgs {
    pub test_targets: Vec<TestTargetWithConfig>,
    pub tests_filter: TestsFilter,
    pub forge_config: Arc<ForgeConfig>,
    pub fork_targets: Vec<ForkTarget>,
    pub package_name: String,
}

pub fn prepare_package(
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
        test_targets: compiled_test_crates
            .into_iter()
            .map(TestCrateRaw::with_config)
            .collect(),
        forge_config,
        tests_filter: test_filter,
        fork_targets: forge_config_from_scarb.fork,
        package_name: package.name,
    })
}

pub async fn run_from_package(
    RunFromCrateArgs {
        test_targets,
        forge_config,
        tests_filter,
        fork_targets,
        package_name,
    }: RunFromCrateArgs,
    block_number_map: &mut BlockNumberMap,
) -> Result<Vec<TestTargetSummary>> {
    let mut test_targets_with_resolved_config: Vec<TestTargetWithResolvedConfig> =
        Vec::with_capacity(test_targets.len());

    for test_target in test_targets {
        let compiled_test_crate =
            resolve_config(test_target, &fork_targets, block_number_map).await?;

        test_targets_with_resolved_config.push(compiled_test_crate);
    }

    let all_tests: usize = test_targets_with_resolved_config
        .iter()
        .map(|tc| tc.test_cases.len())
        .sum();

    let test_targets = test_targets_with_resolved_config
        .into_iter()
        .map(|mut tc| {
            tests_filter.filter_tests(&mut tc.test_cases)?;
            Ok(tc)
        })
        .collect::<Result<Vec<TestTargetWithResolvedConfig>>>()?;
    let not_filtered: usize = test_targets.iter().map(|tc| tc.test_cases.len()).sum();
    let filtered = all_tests - not_filtered;

    warn_if_available_gas_used_with_incompatible_scarb_version(&test_targets)?;
    warn_if_incompatible_rpc_version(&test_targets).await?;

    pretty_printing::print_collected_tests_count(not_filtered, &package_name);

    let mut summaries = vec![];

    for test_target in test_targets {
        pretty_printing::print_running_tests(
            test_target.tests_location,
            test_target.test_cases.len(),
        );

        let forge_config = forge_config.clone();

        let summary =
            run_test_target(test_target, forge_config, &tests_filter, &package_name).await?;

        match summary {
            TestTargetRunResult::Ok(summary) => {
                summaries.push(summary);
            }
            TestTargetRunResult::Interrupted(summary) => {
                summaries.push(summary);
                // Handle scenario for --exit-first flag.
                // Because snforge runs test crates one by one synchronously.
                // In case of test FAIL with --exit-first flag stops processing the next crates
                break;
            }
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
