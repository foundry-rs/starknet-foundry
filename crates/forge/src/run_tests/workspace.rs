use super::package::RunForPackageArgs;
use super::structs::{LatestBlocksNumbersMessage, TestsFailureSummaryMessage};
use crate::warn::{error_if_snforge_std_not_compatible, warn_if_backtrace_without_panic_hint};
use crate::{
    ColorOption, ExitStatus, TestArgs, block_number_map::BlockNumberMap,
    run_tests::package::run_for_package, scarb::build_artifacts_with_scarb,
    shared_cache::FailedTestsCache, warn::warn_if_snforge_std_not_compatible,
};
use anyhow::{Context, Result};
use forge_runner::{CACHE_DIR, test_target_summary::TestTargetSummary};
use forge_runner::{
    coverage_api::can_coverage_be_generated,
    test_case_summary::{AnyTestCaseSummary, TestCaseSummary},
};
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

pub async fn run_for_workspace(args: TestArgs, ui: Arc<UI>) -> Result<ExitStatus> {
    match args.color {
        // SAFETY: This runs in a single-threaded environment.
        ColorOption::Always => unsafe { env::set_var("CLICOLOR_FORCE", "1") },
        // SAFETY: This runs in a single-threaded environment.
        ColorOption::Never => unsafe { env::set_var("CLICOLOR", "0") },
        ColorOption::Auto => (),
    }

    let scarb_metadata = ScarbCommand::metadata().inherit_stderr().run()?;

    if args.coverage {
        can_coverage_be_generated(&scarb_metadata)?;
    }

    error_if_snforge_std_not_compatible(&scarb_metadata)?;
    warn_if_snforge_std_not_compatible(&scarb_metadata, &ui)?;
    warn_if_backtrace_without_panic_hint(&scarb_metadata, &ui);

    let artifacts_dir_path =
        target_dir_for_workspace(&scarb_metadata).join(&scarb_metadata.current_profile);

    let packages: Vec<PackageMetadata> = args
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
        args.features.clone(),
        &scarb_metadata.app_version_info.version,
        args.no_optimization,
    )?;

    let mut block_number_map = BlockNumberMap::default();
    let mut all_failed_tests = vec![];

    let workspace_root = &scarb_metadata.workspace.root;
    let cache_dir = workspace_root.join(CACHE_DIR);
    let trace_verbosity = args.trace_verbosity;

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

        let tests_file_summaries =
            run_for_package(args, &mut block_number_map, trace_verbosity, ui.clone()).await?;

        all_failed_tests.extend(extract_failed_tests(tests_file_summaries));
    }

    FailedTestsCache::new(&cache_dir).save_failed_tests(&all_failed_tests)?;

    let should_print_latest_blocks = !block_number_map.get_url_to_latest_block_number().is_empty();
    if should_print_latest_blocks {
        ui.println(&LatestBlocksNumbersMessage::new(
            block_number_map.get_url_to_latest_block_number().clone(),
        ));
    }
    ui.println(&TestsFailureSummaryMessage::new(&all_failed_tests));

    if args.exact {
        unset_forge_test_filter();
    }

    Ok(if all_failed_tests.is_empty() {
        ExitStatus::Success
    } else {
        ExitStatus::Failure
    })
}

fn extract_failed_tests(
    tests_summaries: Vec<TestTargetSummary>,
) -> impl Iterator<Item = AnyTestCaseSummary> {
    tests_summaries
        .into_iter()
        .flat_map(|test_file_summary| test_file_summary.test_case_summaries)
        .filter(|test_case_summary| {
            matches!(
                test_case_summary,
                AnyTestCaseSummary::Fuzzing(TestCaseSummary::Failed { .. })
                    | AnyTestCaseSummary::Single(TestCaseSummary::Failed { .. })
            )
        })
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
