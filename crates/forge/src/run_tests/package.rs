use super::test_target::{TestTargetRunResult, run_for_test_target};
use crate::run_tests::resolve_config::resolve_config;
use crate::{
    block_number_map::BlockNumberMap,
    run_tests::messages::{
        collected_tests_count::CollectedTestsCountMessage, tests_summary::TestsSummaryMessage,
    },
    scarb::config::ForkTarget,
    test_filter::{NameFilter, TestsFilter},
    warn::warn_if_incompatible_rpc_version,
};
use anyhow::Result;
use console::Style;
use forge_runner::package_tests::TestTargetResolved;
use forge_runner::{
    forge_config::{ForgeConfig, ForgeTrackedResource},
    package_tests::TestTarget,
    test_case_summary::AnyTestCaseSummary,
    test_target_summary::TestTargetSummary,
};
use foundry_ui::{UI, components::labeled::LabeledMessage};
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

#[tracing::instrument(skip_all, level = "debug")]
pub async fn test_package_with_config_resolved(
    test_targets: Vec<TestTarget>,
    fork_targets: &[ForkTarget],
    block_number_map: &mut BlockNumberMap,
    tests_filter: &TestsFilter,
    tracked_resource: &ForgeTrackedResource,
) -> Result<Vec<TestTargetResolved>> {
    let mut test_targets_with_resolved_config = Vec::with_capacity(test_targets.len());

    for test_target in test_targets {
        let test_target = resolve_config(
            test_target,
            fork_targets,
            block_number_map,
            tests_filter,
            tracked_resource,
        )
        .await?;

        test_targets_with_resolved_config.push(test_target);
    }

    Ok(test_targets_with_resolved_config)
}

fn sum_test_cases(test_targets: &[TestTargetResolved]) -> usize {
    test_targets.iter().map(|tc| tc.test_cases.len()).sum()
}

#[tracing::instrument(skip_all, level = "debug")]
pub async fn run_for_package(
    package_name: String,
    forge_config: Arc<ForgeConfig>,
    mut test_targets: Vec<TestTargetResolved>,
    tests_filter: &TestsFilter,
    ui: Arc<UI>,
) -> Result<PackageTestResult> {
    let all_tests = sum_test_cases(&test_targets);

    for test_target in &mut test_targets {
        tests_filter.filter_tests(&mut test_target.test_cases)?;
    }

    warn_if_incompatible_rpc_version(&test_targets, ui.clone()).await?;

    let not_filtered = sum_test_cases(&test_targets);
    ui.println(&CollectedTestsCountMessage {
        tests_num: not_filtered,
        package_name: package_name.clone(),
    });

    let mut summaries = vec![];

    for test_target in test_targets {
        let ui = ui.clone();
        let summary =
            run_for_test_target(test_target, forge_config.clone(), &tests_filter, ui).await?;

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
