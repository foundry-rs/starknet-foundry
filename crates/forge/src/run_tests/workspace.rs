use super::package::RunForPackageArgs;
use crate::run_tests::messages::latest_blocks_numbers::LatestBlocksNumbersMessage;
use crate::run_tests::messages::tests_failure_summary::TestsFailureSummaryMessage;
use crate::run_tests::messages::workspace_summary::WorkspaceSummaryMessage;
use crate::{
    ExitStatus, TestArgs, block_number_map::BlockNumberMap, run_tests::package::run_for_package,
    scarb::build_artifacts_with_scarb, shared_cache::FailedTestsCache,
};
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use forge_runner::test_case_summary::AnyTestCaseSummary;
use forge_runner::{CACHE_DIR, test_target_summary::TestTargetSummary};
use foundry_ui::UI;
use scarb_api::{
    metadata::{Metadata, PackageMetadata},
    target_dir_for_workspace,
};
use scarb_ui::args::PackagesFilter;
use shared::consts::SNFORGE_TEST_FILTER;
use std::env;
use std::sync::Arc;

pub struct WorkspaceDirs {
    pub artifacts_dir: Utf8PathBuf,
    pub cache_dir: Utf8PathBuf,
    pub root_dir: Utf8PathBuf,
}

impl WorkspaceDirs {
    pub fn new(scarb_metadata: &Metadata) -> Self {
        let artifacts_dir =
            target_dir_for_workspace(&scarb_metadata).join(&scarb_metadata.current_profile);
        let workspace_root = &scarb_metadata.workspace.root;
        let cache_dir = workspace_root.join(CACHE_DIR);

        Self {
            artifacts_dir,
            cache_dir,
            root_dir: workspace_root.clone(),
        }
    }
}

#[tracing::instrument(skip_all, level = "debug")]
pub async fn run_for_workspace(
    scarb_metadata: &Metadata,
    args: TestArgs,
    ui: Arc<UI>,
) -> Result<ExitStatus> {
    let workspace_dirs = WorkspaceDirs::new(scarb_metadata);

    let packages: Vec<PackageMetadata> = args
        .scarb_args
        .packages_filter
        .match_many(scarb_metadata)
        .context("Failed to find any packages matching the specified filter")?;

    let filter = PackagesFilter::generate_for::<Metadata>(packages.iter());

    if args.exact {
        let test_filter = args.test_filter.clone();
        if let Some(last_filter) =
            test_filter.and_then(|filter| filter.split("::").last().map(String::from))
        {
            set_forge_test_filter(last_filter);
        }
    }

    build_artifacts_with_scarb(
        filter.clone(),
        args.scarb_args.features.clone(),
        args.scarb_args.profile.clone(),
        args.no_optimization,
    )?;

    let mut block_number_map = BlockNumberMap::default();
    let mut all_tests = vec![];
    let mut total_filtered_count = Some(0);

    let packages_len = packages.len();

    for package in packages {
        env::set_current_dir(&package.root)?;

        let args = RunForPackageArgs::build(package, scarb_metadata, &args, &workspace_dirs, &ui)?;

        let result = run_for_package(args, &mut block_number_map, ui.clone()).await?;

        let filtered = result.filtered();
        all_tests.extend(result.summaries());

        // Accumulate filtered test counts across packages. When using --exact flag,
        // result.filtered_count is None, so total_filtered_count becomes None too.
        total_filtered_count = total_filtered_count
            .zip(filtered)
            .map(|(total, filtered)| total + filtered);
    }

    let workspace_summary = WorkspaceSummaryMessage::new(&all_tests, total_filtered_count);
    let all_failed_tests: Vec<AnyTestCaseSummary> = extract_failed_tests(all_tests).collect();

    FailedTestsCache::new(&workspace_dirs.cache_dir).save_failed_tests(&all_failed_tests)?;

    if !block_number_map.get_url_to_latest_block_number().is_empty() {
        ui.println(&LatestBlocksNumbersMessage::new(
            block_number_map.get_url_to_latest_block_number().clone(),
        ));
    }

    ui.println(&TestsFailureSummaryMessage::new(&all_failed_tests));

    // Print the overall summary only when testing multiple packages
    if packages_len > 1 {
        // Add newline to separate summary from previous output
        ui.print_blank_line();
        ui.println(&workspace_summary);
    }

    if args.exact {
        unset_forge_test_filter();
    }

    Ok(if all_failed_tests.is_empty() {
        ExitStatus::Success
    } else {
        ExitStatus::Failure
    })
}

#[tracing::instrument(skip_all, level = "debug")]
fn extract_failed_tests(
    tests_summaries: Vec<TestTargetSummary>,
) -> impl Iterator<Item = AnyTestCaseSummary> {
    tests_summaries
        .into_iter()
        .flat_map(|summary| summary.test_case_summaries)
        .filter(AnyTestCaseSummary::is_failed)
}

fn set_forge_test_filter(test_filter: String) {
    // SAFETY: This runs in a single-threaded environment.
    unsafe {
        env::set_var(SNFORGE_TEST_FILTER, test_filter);
    };
}

fn unset_forge_test_filter() {
    // SAFETY: This runs in a single-threaded environment.
    unsafe {
        env::remove_var(SNFORGE_TEST_FILTER);
    };
}
