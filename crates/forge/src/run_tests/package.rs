use super::{
    resolve_config::resolve_config,
    structs::{CollectedTestsCountMessage, TestsRunMessage, TestsSummaryMessage},
    test_target::{TestTargetRunResult, run_for_test_target},
};
use crate::{
    TestArgs,
    block_number_map::BlockNumberMap,
    combine_configs::combine_configs,
    scarb::{
        config::{ForgeConfigFromScarb, ForkTarget},
        load_test_artifacts, should_compile_starknet_contract_target,
    },
    shared_cache::FailedTestsCache,
    test_filter::{NameFilter, TestsFilter},
    warn::{
        warn_if_available_gas_used_with_incompatible_scarb_version,
        warn_if_incompatible_rpc_version,
    },
};
use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use configuration::load_package_config;
use console::Style;
use forge_runner::debugging::TraceVerbosity;
use forge_runner::{
    forge_config::ForgeConfig,
    package_tests::{raw::TestTargetRaw, with_config_resolved::TestTargetWithResolvedConfig},
    running::with_config::test_target_with_config,
    test_case_summary::AnyTestCaseSummary,
    test_target_summary::TestTargetSummary,
};
use foundry_ui::{UI, components::labeled::LabeledMessage};
use scarb_api::get_contracts_artifacts_and_source_sierra_paths;
use scarb_metadata::{Metadata, PackageMetadata};
use std::sync::Arc;

pub struct PackageTestResult {
    summaries: Vec<TestTargetSummary>,
    filtered: Option<usize>,
}

impl PackageTestResult {
    #[must_use]
    pub fn new(summaries: Vec<TestTargetSummary>, filtered: Option<usize>) -> Self {
        Self {
            summaries,
            filtered,
        }
    }

    #[must_use]
    pub fn filtered(&self) -> Option<usize> {
        self.filtered
    }

    #[must_use]
    pub fn summaries(self) -> Vec<TestTargetSummary> {
        self.summaries
    }
}

pub struct RunForPackageArgs {
    pub test_targets: Vec<TestTargetRaw>,
    pub tests_filter: TestsFilter,
    pub forge_config: Arc<ForgeConfig>,
    pub fork_targets: Vec<ForkTarget>,
    pub package_name: String,
}

impl RunForPackageArgs {
    pub fn build(
        package: PackageMetadata,
        scarb_metadata: &Metadata,
        args: &TestArgs,
        cache_dir: &Utf8PathBuf,
        artifacts_dir: &Utf8Path,
        ui: &UI,
    ) -> Result<RunForPackageArgs> {
        let raw_test_targets = load_test_artifacts(artifacts_dir, &package)?;

        let contracts = get_contracts_artifacts_and_source_sierra_paths(
            artifacts_dir,
            &package,
            !should_compile_starknet_contract_target(
                &scarb_metadata.app_version_info.version,
                args.no_optimization,
            ),
            ui,
        )?;
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
            args.coverage,
            args.max_n_steps,
            args.tracked_resource,
            contracts_data,
            cache_dir.clone(),
            &forge_config_from_scarb,
            &args.additional_args,
        ));

        let test_filter = TestsFilter::from_flags(
            args.test_filter.clone(),
            args.exact,
            args.skip.clone(),
            args.only_ignored,
            args.include_ignored,
            args.rerun_failed,
            FailedTestsCache::new(cache_dir),
        );

        Ok(RunForPackageArgs {
            test_targets: raw_test_targets,
            forge_config,
            tests_filter: test_filter,
            fork_targets: forge_config_from_scarb.fork,
            package_name: package.name,
        })
    }
}

async fn test_package_with_config_resolved(
    test_targets: Vec<TestTargetRaw>,
    fork_targets: &[ForkTarget],
    block_number_map: &mut BlockNumberMap,
    forge_config: &ForgeConfig,
) -> Result<Vec<TestTargetWithResolvedConfig>> {
    let mut test_targets_with_resolved_config = Vec::with_capacity(test_targets.len());

    for test_target in test_targets {
        let test_target = test_target_with_config(
            test_target,
            &forge_config.test_runner_config.tracked_resource,
        )?;

        let test_target = resolve_config(test_target, fork_targets, block_number_map).await?;

        test_targets_with_resolved_config.push(test_target);
    }

    Ok(test_targets_with_resolved_config)
}

fn sum_test_cases(test_targets: &[TestTargetWithResolvedConfig]) -> usize {
    test_targets.iter().map(|tc| tc.test_cases.len()).sum()
}

pub async fn run_for_package(
    RunForPackageArgs {
        test_targets,
        forge_config,
        tests_filter,
        fork_targets,
        package_name,
    }: RunForPackageArgs,
    block_number_map: &mut BlockNumberMap,
    trace_verbosity: Option<TraceVerbosity>,
    ui: Arc<UI>,
) -> Result<PackageTestResult> {
    let mut test_targets = test_package_with_config_resolved(
        test_targets,
        &fork_targets,
        block_number_map,
        &forge_config,
    )
    .await?;
    let all_tests = sum_test_cases(&test_targets);

    for test_target in &mut test_targets {
        tests_filter.filter_tests(&mut test_target.test_cases)?;
    }

    warn_if_available_gas_used_with_incompatible_scarb_version(&test_targets, &ui)?;
    warn_if_incompatible_rpc_version(&test_targets, ui.clone()).await?;

    let not_filtered = sum_test_cases(&test_targets);
    ui.println(&CollectedTestsCountMessage {
        tests_num: not_filtered,
        package_name: package_name.clone(),
    });

    let mut summaries = vec![];

    for test_target in test_targets {
        let ui = ui.clone();
        ui.println(&TestsRunMessage::new(
            test_target.tests_location,
            test_target.test_cases.len(),
        ));

        let summary = run_for_test_target(
            test_target,
            forge_config.clone(),
            &tests_filter,
            trace_verbosity,
            ui,
        )
        .await?;

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

    // TODO(#2574): Bring back "filtered out" number in tests summary when running with `--exact` flag
    let filtered_count = if let NameFilter::ExactMatch(_) = tests_filter.name_filter {
        None
    } else {
        Some(all_tests - not_filtered)
    };

    ui.println(&TestsSummaryMessage::new(&summaries, filtered_count));

    let any_fuzz_test_was_run = summaries.iter().any(|test_target_summary| {
        test_target_summary
            .test_case_summaries
            .iter()
            .filter(|summary| matches!(summary, AnyTestCaseSummary::Fuzzing(_)))
            .any(|summary| summary.is_passed() || summary.is_failed())
    });

    if any_fuzz_test_was_run {
        ui.println(&LabeledMessage::new(
            &Style::new().bold().apply_to("Fuzzer seed").to_string(),
            &forge_config.test_runner_config.fuzzer_seed.to_string(),
        ));
    }

    Ok(PackageTestResult::new(summaries, filtered_count))
}
