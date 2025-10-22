use super::package::RunForPackageArgs;
use crate::partition::PartitionConfig;
use crate::profile_validation::check_profile_compatibility;
use crate::run_tests::messages::latest_blocks_numbers::LatestBlocksNumbersMessage;
use crate::run_tests::messages::overall_summary::OverallSummaryMessage;
use crate::run_tests::messages::partition::PartitionMessage;
use crate::run_tests::messages::tests_failure_summary::TestsFailureSummaryMessage;
use crate::warn::{
    error_if_snforge_std_deprecated_missing, error_if_snforge_std_deprecated_not_compatible,
    error_if_snforge_std_not_compatible, warn_if_backtrace_without_panic_hint,
    warn_if_snforge_std_deprecated_does_not_match_package_version,
};
use crate::{
    ColorOption, ExitStatus, MINIMAL_SCARB_VERSION_FOR_V2_MACROS_REQUIREMENT, TestArgs,
    block_number_map::BlockNumberMap, run_tests::package::run_for_package,
    scarb::build_artifacts_with_scarb, shared_cache::FailedTestsCache,
    warn::warn_if_snforge_std_does_not_match_package_version,
};
use anyhow::{Context, Result};
use forge_runner::test_case_summary::AnyTestCaseSummary;
use forge_runner::{CACHE_DIR, test_target_summary::TestTargetSummary};
use foundry_ui::UI;
use scarb_api::{
    ScarbCommand,
    metadata::{Metadata, MetadataCommandExt, PackageMetadata},
    target_dir_for_workspace,
};
use scarb_ui::args::PackagesFilter;
use shared::consts::SNFORGE_TEST_FILTER;
use std::env;
use std::sync::Arc;

#[tracing::instrument(skip_all, level = "debug")]
#[expect(clippy::too_many_lines)]
pub async fn run_for_workspace(args: TestArgs, ui: Arc<UI>) -> Result<ExitStatus> {
    match args.color {
        // SAFETY: This runs in a single-threaded environment.
        ColorOption::Always => unsafe { env::set_var("CLICOLOR_FORCE", "1") },
        // SAFETY: This runs in a single-threaded environment.
        ColorOption::Never => unsafe { env::set_var("CLICOLOR", "0") },
        ColorOption::Auto => (),
    }

    let mut metadata_command = ScarbCommand::metadata();
    if let Some(profile) = &args.scarb_args.profile.specified() {
        metadata_command.profile(profile.clone());
    }
    let scarb_metadata = metadata_command.inherit_stderr().run()?;

    check_profile_compatibility(&args, &scarb_metadata)?;

    let scarb_version = ScarbCommand::version().run()?.scarb;
    if scarb_version >= MINIMAL_SCARB_VERSION_FOR_V2_MACROS_REQUIREMENT {
        error_if_snforge_std_not_compatible(&scarb_metadata)?;
        warn_if_snforge_std_does_not_match_package_version(&scarb_metadata, &ui)?;
    } else {
        error_if_snforge_std_deprecated_missing(&scarb_metadata)?;
        error_if_snforge_std_deprecated_not_compatible(&scarb_metadata)?;
        warn_if_snforge_std_deprecated_does_not_match_package_version(&scarb_metadata, &ui)?;
    }

    warn_if_backtrace_without_panic_hint(&scarb_metadata, &ui);

    let artifacts_dir_path =
        target_dir_for_workspace(&scarb_metadata).join(&scarb_metadata.current_profile);

    let packages: Vec<PackageMetadata> = args
        .scarb_args
        .packages_filter
        .match_many(&scarb_metadata)
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
        &scarb_metadata.app_version_info.version,
        args.no_optimization,
    )?;

    let mut block_number_map = BlockNumberMap::default();
    let mut all_tests = vec![];
    let mut total_filtered_count = Some(0);

    let workspace_root = &scarb_metadata.workspace.root;
    let cache_dir = workspace_root.join(CACHE_DIR);
    let packages_len = packages.len();

    if let Some(partition) = &args.partition {
        // When partitioning is used, we need to collect all tests across all packages first
        // to create a consistent mapping of tests to partitions.
        // This ensures that partitions are equal, i.e. difference in number of tests is <= 1).
        let packages_args = packages
            .iter()
            .map(|package| {
                RunForPackageArgs::build(
                    package.clone(),
                    &scarb_metadata,
                    &args,
                    &cache_dir,
                    &artifacts_dir_path,
                    &ui,
                )
            })
            .collect::<Result<Vec<RunForPackageArgs>>>()?;

        let partition_config = PartitionConfig::new(*partition, &packages_args);

        for (package, args) in packages.iter().zip(packages_args) {
            env::set_current_dir(&package.root)?;

            let result = run_for_package(
                args,
                &mut block_number_map,
                Some(&partition_config),
                ui.clone(),
            )
            .await?;

            let filtered = result.filtered();
            all_tests.extend(result.summaries());

            total_filtered_count = calculate_total_filtered_count(total_filtered_count, filtered);
        }
    } else {
        for package in packages {
            env::set_current_dir(&package.root)?;

            let args = RunForPackageArgs::build(
                package,
                &scarb_metadata,
                &args,
                &cache_dir,
                &artifacts_dir_path,
                &ui,
            )?;

            let result = run_for_package(args, &mut block_number_map, None, ui.clone()).await?;

            let filtered = result.filtered();
            all_tests.extend(result.summaries());

            total_filtered_count = calculate_total_filtered_count(total_filtered_count, filtered);
        }
    }

    let overall_summary = OverallSummaryMessage::new(&all_tests, total_filtered_count);

    let all_failed_tests: Vec<AnyTestCaseSummary> = extract_failed_tests(all_tests).collect();

    FailedTestsCache::new(&cache_dir).save_failed_tests(&all_failed_tests)?;

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
        ui.println(&overall_summary);
    }

    if let Some(partition) = &args.partition {
        ui.println(&PartitionMessage::new(*partition));
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

fn calculate_total_filtered_count(
    total_filtered_count: Option<usize>,
    filtered: Option<usize>,
) -> Option<usize> {
    // Accumulate filtered test counts across packages. When using `--exact` flag,
    // `result.filtered_count` is None, so `total_filtered_count` becomes None too.
    total_filtered_count
        .zip(filtered)
        .map(|(total, filtered)| total + filtered)
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
