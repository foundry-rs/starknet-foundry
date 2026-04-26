use super::{
    resolve_config::resolve_config,
    test_target::{ExitFirstChannel, TestTargetRunResult, run_for_test_target},
};
use crate::scarb::{
    config::{ForgeConfigFromScarb, ForkTarget},
    load_package_config,
};
use crate::{
    TestArgs,
    block_number_map::BlockNumberMap,
    combine_configs::combine_configs,
    run_tests::messages::{
        collected_tests_count::CollectedTestsCountMessage, tests_run::TestsRunMessage,
        tests_summary::TestsSummaryMessage,
    },
    shared_cache::FailedTestsCache,
    test_filter::{NameFilter, TestsFilter},
    warn::warn_if_incompatible_rpc_version,
};
use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use console::Style;
use forge_runner::{
    forge_config::{ForgeConfig, ForgeTrackedResource},
    package_tests::{
        raw::TestTargetRaw,
        with_config::TestTargetWithConfig,
        with_config_resolved::{TestCaseWithResolvedConfig, sanitize_test_case_name},
    },
    partition::PartitionConfig,
    running::target::prepare_test_target,
    scarb::load_test_artifacts,
    test_case_summary::AnyTestCaseSummary,
    test_target_summary::TestTargetSummary,
};
use foundry_ui::{UI, components::labeled::LabeledMessage};
use scarb_api::{CompilationOpts, get_contracts_artifacts_and_source_sierra_paths};
use scarb_metadata::{Metadata, PackageMetadata};
use std::sync::Arc;
use tokio::task::JoinHandle;

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
    pub target_handles: Vec<JoinHandle<Result<TestTargetWithConfig>>>,
    pub tests_filter: TestsFilter,
    pub forge_config: Arc<ForgeConfig>,
    pub fork_targets: Vec<ForkTarget>,
    pub package_name: String,
    pub package_root: Utf8PathBuf,
}

impl RunForPackageArgs {
    #[tracing::instrument(skip_all, level = "debug")]
    #[expect(clippy::too_many_arguments)]
    pub fn build(
        package: PackageMetadata,
        scarb_metadata: &Metadata,
        args: &TestArgs,
        cache_dir: &Utf8PathBuf,
        artifacts_dir: &Utf8Path,
        partitioning_config: PartitionConfig,
        predeployed_contracts: Option<&ContractsData>,
        ui: &UI,
    ) -> Result<RunForPackageArgs> {
        let mut raw_test_targets = load_test_artifacts(artifacts_dir, &package)?;

        let contracts = get_contracts_artifacts_and_source_sierra_paths(
            artifacts_dir,
            &package,
            ui,
            CompilationOpts {
                use_test_target_contracts: !args.no_optimization,
                #[cfg(feature = "cairo-native")]
                run_native: args.run_native,
            },
        )?;
        let mut contracts_data = ContractsData::try_from(contracts)?;
        if let Some(predeployed_contracts) = predeployed_contracts {
            contracts_data.try_extend(predeployed_contracts)?;
        }

        let forge_config_from_scarb =
            load_package_config::<ForgeConfigFromScarb>(scarb_metadata, &package.id)?;
        let forge_config = Arc::new(combine_configs(
            args,
            contracts_data,
            cache_dir.clone(),
            &forge_config_from_scarb,
        ));

        let tests_filter = TestsFilter::from_flags(
            args.test_filter.clone(),
            args.exact,
            args.skip.clone(),
            args.only_ignored,
            args.include_ignored,
            args.rerun_failed,
            FailedTestsCache::new(cache_dir),
            partitioning_config,
        );

        if args.deterministic_output {
            raw_test_targets.sort_by_key(|t| t.tests_location);
        }

        let tracked_resource = forge_config.test_runner_config.tracked_resource;

        let target_handles = raw_test_targets
            .into_iter()
            .map(|t| spawn_prepare_test_target(t, tracked_resource))
            .collect();

        Ok(RunForPackageArgs {
            target_handles,
            forge_config,
            tests_filter,
            fork_targets: forge_config_from_scarb.fork,
            package_name: package.name.clone(),
            package_root: package.root,
        })
    }
}

fn spawn_prepare_test_target(
    target: TestTargetRaw,
    tracked_resource: ForgeTrackedResource,
) -> JoinHandle<Result<TestTargetWithConfig>> {
    tokio::task::spawn_blocking(move || prepare_test_target(target, &tracked_resource))
}

fn sum_test_cases_from_test_target(
    test_cases: &[TestCaseWithResolvedConfig],
    partitioning_config: &PartitionConfig,
) -> usize {
    match partitioning_config {
        PartitionConfig::Disabled => test_cases.len(),
        PartitionConfig::Enabled {
            partition,
            partition_map,
        } => test_cases
            .iter()
            .filter(|test_case| {
                let test_assigned_index = partition_map
                    .get_assigned_index(&sanitize_test_case_name(&test_case.name))
                    .expect("Partition map must contain all test cases");
                test_assigned_index == partition.index()
            })
            .count(),
    }
}

#[tracing::instrument(skip_all, level = "debug")]
pub async fn run_for_package(
    RunForPackageArgs {
        target_handles,
        forge_config,
        tests_filter,
        fork_targets,
        package_name,
        package_root: _,
    }: RunForPackageArgs,
    block_number_map: &BlockNumberMap,
    ui: Arc<UI>,
    exit_first_channel: &mut ExitFirstChannel,
) -> Result<PackageTestResult> {
    // Resolve all targets first so the collected count includes #[ignore] filtering.
    let mut resolved_targets = vec![];
    let mut all_tests = 0;
    let mut not_filtered_total = 0;

    for handle in target_handles {
        let target_with_config = handle.await??;

        let mut resolved = resolve_config(
            target_with_config,
            &fork_targets,
            block_number_map,
            &tests_filter,
        )
        .await?;

        let all = sum_test_cases_from_test_target(
            &resolved.test_cases,
            &tests_filter.partitioning_config,
        );
        tests_filter.filter_tests(&mut resolved.test_cases)?;
        let not_filtered = sum_test_cases_from_test_target(
            &resolved.test_cases,
            &tests_filter.partitioning_config,
        );
        all_tests += all;
        not_filtered_total += not_filtered;

        resolved_targets.push(resolved);
    }

    warn_if_incompatible_rpc_version(&resolved_targets, ui.clone()).await?;

    ui.println(&CollectedTestsCountMessage {
        tests_num: not_filtered_total,
        package_name: package_name.clone(),
    });

    let mut summaries = vec![];

    for resolved in resolved_targets {
        ui.println(&TestsRunMessage::new(
            resolved.tests_location,
            sum_test_cases_from_test_target(
                &resolved.test_cases,
                &tests_filter.partitioning_config,
            ),
        ));

        let summary = run_for_test_target(
            resolved,
            forge_config.clone(),
            &tests_filter,
            ui.clone(),
            exit_first_channel,
        )
        .await?;

        let (TestTargetRunResult::Ok(s) | TestTargetRunResult::Interrupted(s)) = summary;
        summaries.push(s);
    }

    // TODO(#2574): Bring back "filtered out" number in tests summary when running with `--exact` flag
    let filtered_count = if let NameFilter::ExactMatch(_) = tests_filter.name_filter {
        None
    } else {
        Some(all_tests - not_filtered_total)
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
