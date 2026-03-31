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
    warn::warn_if_incompatible_rpc_version_for_target,
};
use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use console::Style;
use forge_runner::{
    forge_config::ForgeConfig,
    package_tests::{
        raw::TestTargetRaw,
        with_config_resolved::{
            TestCaseWithResolvedConfig, TestTargetWithResolvedConfig, sanitize_test_case_name,
        },
    },
    partition::PartitionConfig,
    running::with_config::test_target_with_config,
    scarb::load_test_artifacts,
    test_case_summary::AnyTestCaseSummary,
    test_target_summary::TestTargetSummary,
};
use foundry_ui::{UI, components::labeled::LabeledMessage};
use scarb_api::{CompilationOpts, get_contracts_artifacts_and_source_sierra_paths};
use scarb_metadata::{Metadata, PackageMetadata};
use std::collections::HashSet;
use std::sync::Arc;
use url::Url;

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
    #[tracing::instrument(skip_all, level = "debug")]
    pub fn build(
        package: PackageMetadata,
        scarb_metadata: &Metadata,
        args: &TestArgs,
        cache_dir: &Utf8PathBuf,
        artifacts_dir: &Utf8Path,
        partitioning_config: PartitionConfig,
        ui: &UI,
    ) -> Result<RunForPackageArgs> {
        let raw_test_targets = load_test_artifacts(artifacts_dir, &package)?;

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
        let contracts_data = ContractsData::try_from(contracts)?;

        let forge_config_from_scarb =
            load_package_config::<ForgeConfigFromScarb>(scarb_metadata, &package.id)?;
        let forge_config = Arc::new(combine_configs(
            args.exit_first,
            args.deterministic_output,
            args.fuzzer_runs,
            args.fuzzer_seed,
            args.detailed_resources,
            args.save_trace_data,
            args.build_profile,
            args.coverage,
            args.gas_report,
            args.max_n_steps,
            args.tracked_resource,
            contracts_data,
            cache_dir.clone(),
            &forge_config_from_scarb,
            &args.additional_args,
            args.trace_args.clone(),
        ));

        let test_filter = TestsFilter::from_flags(
            args.test_filter.clone(),
            args.exact,
            args.skip.clone(),
            args.only_ignored,
            args.include_ignored,
            args.rerun_failed,
            FailedTestsCache::new(cache_dir),
            partitioning_config,
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
async fn prepare_test_target(
    raw: TestTargetRaw,
    fork_targets: &[ForkTarget],
    block_number_map: &mut BlockNumberMap,
    forge_config: &ForgeConfig,
    tests_filter: &TestsFilter,
    warned_urls: &mut HashSet<Url>,
    ui: Arc<UI>,
) -> Result<(TestTargetWithResolvedConfig, usize, usize)> {
    let test_target = test_target_with_config(
        raw,
        &forge_config.test_runner_config.tracked_resource,
    )?;

    let mut test_target =
        resolve_config(test_target, fork_targets, block_number_map, tests_filter).await?;

    let before_filter =
        sum_test_cases_from_test_target(&test_target.test_cases, &tests_filter.partitioning_config);

    tests_filter.filter_tests(&mut test_target.test_cases)?;

    let after_filter =
        sum_test_cases_from_test_target(&test_target.test_cases, &tests_filter.partitioning_config);

    warn_if_incompatible_rpc_version_for_target(&test_target, warned_urls, ui).await?;

    Ok((test_target, before_filter, after_filter))
}

#[tracing::instrument(skip_all, level = "debug")]
pub async fn run_for_package(
    RunForPackageArgs {
        test_targets,
        forge_config,
        tests_filter,
        fork_targets,
        package_name,
    }: RunForPackageArgs,
    block_number_map: &mut BlockNumberMap,
    ui: Arc<UI>,
    exit_first_channel: &mut ExitFirstChannel,
    deterministic_output: bool,
) -> Result<PackageTestResult> {
    let mut warned_urls = HashSet::<Url>::new();
    let mut all_tests: usize = 0;
    let mut not_filtered: usize = 0;
    let mut summaries = vec![];

    let mut test_targets = test_targets;
    if deterministic_output {
        test_targets.sort_by(|a, b| a.tests_location.cmp(&b.tests_location));
    }

    // Prepare the first target eagerly so we can start executing it
    // while preparing subsequent targets.
    let mut prepared: Option<TestTargetWithResolvedConfig> = None;
    let mut remaining = test_targets.into_iter();

    if let Some(first_raw) = remaining.next() {
        let (target, before, after) = prepare_test_target(
            first_raw,
            &fork_targets,
            block_number_map,
            &forge_config,
            &tests_filter,
            &mut warned_urls,
            ui.clone(),
        )
        .await?;
        all_tests += before;
        not_filtered += after;
        prepared = Some(target);
    }

    for next_raw in remaining {
        // Execute the already-prepared target while preparing the next one.
        if let Some(test_target) = prepared.take() {
            ui.println(&TestsRunMessage::new(
                test_target.tests_location,
                sum_test_cases_from_test_target(
                    &test_target.test_cases,
                    &tests_filter.partitioning_config,
                ),
            ));

            let (summary, prep_result) = tokio::join!(
                run_for_test_target(
                    test_target,
                    forge_config.clone(),
                    &tests_filter,
                    ui.clone(),
                    exit_first_channel,
                ),
                prepare_test_target(
                    next_raw,
                    &fork_targets,
                    block_number_map,
                    &forge_config,
                    &tests_filter,
                    &mut warned_urls,
                    ui.clone(),
                )
            );

            let summary = summary?;
            let (next_target, before, after) = prep_result?;

            all_tests += before;
            not_filtered += after;

            let (TestTargetRunResult::Ok(s) | TestTargetRunResult::Interrupted(s)) = summary;
            summaries.push(s);
            prepared = Some(next_target);
        }
    }

    // Execute the last prepared target.
    if let Some(test_target) = prepared.take() {
        ui.println(&TestsRunMessage::new(
            test_target.tests_location,
            sum_test_cases_from_test_target(
                &test_target.test_cases,
                &tests_filter.partitioning_config,
            ),
        ));

        let summary = run_for_test_target(
            test_target,
            forge_config.clone(),
            &tests_filter,
            ui.clone(),
            exit_first_channel,
        )
        .await?;

        let (TestTargetRunResult::Ok(s) | TestTargetRunResult::Interrupted(s)) = summary;
        summaries.push(s);
    }

    ui.println(&CollectedTestsCountMessage {
        tests_num: not_filtered,
        package_name: package_name.clone(),
    });

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
