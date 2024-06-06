use super::run_crate::prepare_crate;
use crate::{
    block_number_map::BlockNumberMap,
    pretty_printing,
    run_tests::run_crate::run_from_crate,
    scarb::{build_contracts_with_scarb, build_test_artifacts_with_scarb},
    shared_cache::FailedTestsCache,
    warn::warn_if_snforge_std_not_compatible,
    ColorOption, ExitStatus, TestArgs,
};
use anyhow::{Context, Result};
use forge_runner::{
    build_trace_data::test_sierra_program_path::VERSIONED_PROGRAMS_DIR,
    test_case_summary::{AnyTestCaseSummary, TestCaseSummary},
};
use forge_runner::{test_crate_summary::TestCrateSummary, CACHE_DIR};
use scarb_api::{
    metadata::{Metadata, MetadataCommandExt, PackageMetadata},
    target_dir_for_workspace, ScarbCommand,
};
use scarb_ui::args::PackagesFilter;
use std::env;

#[allow(clippy::too_many_lines)]
pub async fn prepare_and_run_workspace(args: TestArgs) -> Result<ExitStatus> {
    match args.color {
        ColorOption::Always => env::set_var("CLICOLOR_FORCE", "1"),
        ColorOption::Never => env::set_var("CLICOLOR", "0"),
        ColorOption::Auto => (),
    }

    let scarb_metadata = ScarbCommand::metadata().inherit_stderr().run()?;
    warn_if_snforge_std_not_compatible(&scarb_metadata)?;

    let snforge_target_dir_path = target_dir_for_workspace(&scarb_metadata)
        .join(&scarb_metadata.current_profile)
        .join("snforge");

    let packages: Vec<PackageMetadata> = args
        .packages_filter
        .match_many(&scarb_metadata)
        .context("Failed to find any packages matching the specified filter")?;

    let filter = PackagesFilter::generate_for::<Metadata>(packages.iter());

    build_test_artifacts_with_scarb(filter.clone())?;
    build_contracts_with_scarb(filter)?;

    let mut block_number_map = BlockNumberMap::default();
    let mut all_failed_tests = vec![];

    let workspace_root = &scarb_metadata.workspace.root;
    let cache_dir = workspace_root.join(CACHE_DIR);
    let versioned_programs_dir = workspace_root.join(VERSIONED_PROGRAMS_DIR);

    for package in packages {
        let run_from_crate_args = prepare_crate(
            package,
            &scarb_metadata,
            &args,
            cache_dir.clone(),
            &snforge_target_dir_path,
            versioned_programs_dir.clone(),
        )?;

        let tests_file_summaries =
            run_from_crate(run_from_crate_args, &mut block_number_map).await?;

        all_failed_tests.extend(extract_failed_tests(tests_file_summaries));
    }

    FailedTestsCache::new(cache_dir).save_failed_tests(&all_failed_tests)?;

    pretty_printing::print_latest_blocks_numbers(block_number_map.get_url_to_latest_block_number());
    pretty_printing::print_failures(&all_failed_tests);

    Ok(if all_failed_tests.is_empty() {
        ExitStatus::Success
    } else {
        ExitStatus::Failure
    })
}

fn extract_failed_tests(
    tests_summaries: Vec<TestCrateSummary>,
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
