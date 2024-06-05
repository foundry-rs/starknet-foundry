use crate::{
    block_number_map::BlockNumberMap,
    combine_configs::combine_configs,
    pretty_printing,
    scarb::{
        build_contracts_with_scarb, build_test_artifacts_with_scarb, config::ForgeConfigFromScarb,
        get_test_artifacts_path, load_test_artifacts,
    },
    shared_cache::FailedTestsCache,
    test::run,
    test_filter::TestsFilter,
    warn::warn_if_snforge_std_not_compatible,
    ColorOption, ExitStatus, TestArgs,
};
use anyhow::{Context, Result};
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use configuration::load_package_config;
use forge_runner::{
    build_trace_data::test_sierra_program_path::VERSIONED_PROGRAMS_DIR,
    test_case_summary::{AnyTestCaseSummary, TestCaseSummary},
};
use forge_runner::{test_crate_summary::TestCrateSummary, CACHE_DIR};
use scarb_api::{
    get_contracts_artifacts_and_source_sierra_paths,
    metadata::{Metadata, MetadataCommandExt, PackageMetadata},
    target_dir_for_workspace, ScarbCommand,
};
use scarb_ui::args::PackagesFilter;
use std::env;
use std::sync::Arc;
use std::thread::available_parallelism;
use tokio::runtime::Builder;

#[allow(clippy::too_many_lines)]
pub fn test_workspace(args: TestArgs) -> Result<ExitStatus> {
    match args.color {
        ColorOption::Always => env::set_var("CLICOLOR_FORCE", "1"),
        ColorOption::Never => env::set_var("CLICOLOR", "0"),
        ColorOption::Auto => (),
    }

    let scarb_metadata = ScarbCommand::metadata().inherit_stderr().run()?;
    warn_if_snforge_std_not_compatible(&scarb_metadata)?;

    let workspace_root = scarb_metadata.workspace.root.clone();
    let snforge_target_dir_path = target_dir_for_workspace(&scarb_metadata)
        .join(&scarb_metadata.current_profile)
        .join("snforge");

    let packages: Vec<PackageMetadata> = args
        .packages_filter
        .match_many(&scarb_metadata)
        .context("Failed to find any packages matching the specified filter")?;

    let filter = PackagesFilter::generate_for::<Metadata>(packages.iter());

    build_test_artifacts_with_scarb(filter.clone())?;
    build_contracts_with_scarb(filter.clone())?;

    let cores = if let Ok(available_cores) = available_parallelism() {
        available_cores.get()
    } else {
        eprintln!("Failed to get the number of available cores, defaulting to 1");
        1
    };

    let rt = Builder::new_multi_thread()
        .max_blocking_threads(cores)
        .enable_all()
        .build()?;

    let all_failed_tests = rt.block_on({
        rt.spawn(async move {
            let mut block_number_map = BlockNumberMap::default();
            let mut all_failed_tests = vec![];

            let cache_dir = workspace_root.join(CACHE_DIR);
            let versioned_programs_dir = workspace_root.join(VERSIONED_PROGRAMS_DIR);

            for package in &packages {
                env::set_current_dir(&package.root)?;

                let test_artifacts_path =
                    get_test_artifacts_path(&snforge_target_dir_path, &package.name);
                let compiled_test_crates = load_test_artifacts(&test_artifacts_path)?;

                let contracts = get_contracts_artifacts_and_source_sierra_paths(
                    &scarb_metadata,
                    &package.id,
                    None,
                )?;
                let contracts_data = ContractsData::try_from(contracts)?;

                let forge_config_from_scarb =
                    load_package_config::<ForgeConfigFromScarb>(&scarb_metadata, &package.id)?;
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
                    versioned_programs_dir.clone(),
                    &forge_config_from_scarb,
                ));

                let test_filter = TestsFilter::from_flags(
                    args.test_filter.clone(),
                    args.exact,
                    args.only_ignored,
                    args.include_ignored,
                    args.rerun_failed,
                    FailedTestsCache::new(cache_dir.clone()),
                );

                let tests_file_summaries = run(
                    compiled_test_crates,
                    &package.name,
                    &test_filter,
                    forge_config,
                    &forge_config_from_scarb.fork,
                    &mut block_number_map,
                )
                .await?;

                all_failed_tests.extend(extract_failed_tests(tests_file_summaries));
            }

            FailedTestsCache::new(cache_dir).save_failed_tests(&all_failed_tests)?;

            pretty_printing::print_latest_blocks_numbers(
                block_number_map.get_url_to_latest_block_number(),
            );

            Ok::<_, anyhow::Error>(all_failed_tests)
        })
    })??;

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
