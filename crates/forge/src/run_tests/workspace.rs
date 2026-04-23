use super::package::RunForPackageArgs;
use crate::profile_validation::check_compiler_config_compatibility;
use crate::run_tests::messages::latest_blocks_numbers::LatestBlocksNumbersMessage;
use crate::run_tests::messages::overall_summary::OverallSummaryMessage;
use crate::run_tests::messages::partition::{PartitionFinishedMessage, PartitionStartedMessage};
use crate::run_tests::messages::tests_failure_summary::TestsFailureSummaryMessage;
use crate::warn::{
    error_if_snforge_std_not_compatible, warn_if_snforge_std_does_not_match_package_version,
};
use crate::{
    ColorOption, ExitStatus, TestArgs, block_number_map::BlockNumberMap,
    run_tests::package::run_for_package, run_tests::test_target::ExitFirstChannel,
    scarb::build_artifacts_with_scarb, shared_cache::FailedTestsCache,
};
use anyhow::Result;
use cheatnet::predeployment::contracts_data::load_predeployed_contracts;
use forge_runner::backtrace::is_backtrace_enabled;
use forge_runner::debugging::TraceArgs;
use forge_runner::partition::PartitionConfig;
use forge_runner::test_case_summary::AnyTestCaseSummary;
use forge_runner::{CACHE_DIR, test_target_summary::TestTargetSummary};
use foundry_ui::UI;
use scarb_api::metadata::{MetadataOpts, metadata_with_opts};
use scarb_api::{
    metadata::{Metadata, PackageMetadata},
    packages_from_filter, target_dir_for_workspace,
};
use scarb_ui::args::PackagesFilter;
use shared::consts::SNFORGE_TEST_FILTER;
use std::env;
use std::sync::Arc;

#[derive(Debug)]
pub struct WorkspaceExecutionSummary {
    pub all_tests: Vec<TestTargetSummary>,
}

#[tracing::instrument(skip_all, level = "debug")]
#[allow(clippy::too_many_lines)]
pub async fn run_for_workspace(args: TestArgs, ui: Arc<UI>) -> Result<ExitStatus> {
    let scarb_metadata = metadata_with_opts(MetadataOpts {
        profile: args.scarb_args.profile.specified(),
        ..MetadataOpts::default()
    })?;

    let packages: Vec<PackageMetadata> =
        packages_from_filter(&scarb_metadata, &args.scarb_args.packages_filter)?;

    let filter = PackagesFilter::generate_for::<Metadata>(packages.iter());

    build_artifacts_with_scarb(
        filter.clone(),
        args.scarb_args.features.clone(),
        args.scarb_args.profile.clone(),
        args.no_optimization,
    )?;

    let WorkspaceExecutionSummary { all_tests } =
        execute_workspace(&args, ui, &scarb_metadata).await?;
    let has_failures = extract_failed_tests(&all_tests).next().is_some();
    Ok(if has_failures {
        ExitStatus::Failure
    } else {
        ExitStatus::Success
    })
}

pub async fn execute_workspace(
    args: &TestArgs,
    ui: Arc<UI>,
    scarb_metadata: &Metadata,
) -> Result<WorkspaceExecutionSummary> {
    let deterministic_output = args.deterministic_output;
    match args.color {
        // SAFETY: This runs in a single-threaded environment.
        ColorOption::Always => unsafe { env::set_var("CLICOLOR_FORCE", "1") },
        // SAFETY: This runs in a single-threaded environment.
        ColorOption::Never => unsafe { env::set_var("CLICOLOR", "0") },
        ColorOption::Auto => (),
    }

    let packages: Vec<PackageMetadata> =
        packages_from_filter(scarb_metadata, &args.scarb_args.packages_filter)?;

    check_compiler_config_compatibility(args, scarb_metadata, &packages)?;

    error_if_snforge_std_not_compatible(scarb_metadata)?;
    warn_if_snforge_std_does_not_match_package_version(scarb_metadata, &ui)?;

    let artifacts_dir_path =
        target_dir_for_workspace(scarb_metadata).join(&scarb_metadata.current_profile);

    if args.exact {
        let test_filter = args.test_filter.clone();
        if let Some(last_filter) =
            test_filter.and_then(|filter| filter.split("::").last().map(String::from))
        {
            set_forge_test_filter(last_filter);
        }
    }

    let block_number_map = BlockNumberMap::default();
    let mut all_tests = vec![];
    let mut total_filtered_count = Some(0);
    let mut exit_first_channel = ExitFirstChannel::new();

    let workspace_root = &scarb_metadata.workspace.root;
    let cache_dir = workspace_root.join(CACHE_DIR);
    let packages_len = packages.len();

    let partitioning_config = get_partitioning_config(args, &ui, &packages, &artifacts_dir_path)?;

    // Load predeployed contracts if backtrace is enabled or any trace-related arguments are provided.
    let predeployed_contracts = should_load_predeployed_contracts_sierra(&args.trace_args)
        .then(load_predeployed_contracts)
        .transpose()?;

    // Spawn config passes for all packages before running any tests so that
    // compilation overlaps with test execution across packages.
    let mut all_package_args = Vec::with_capacity(packages.len());
    for pkg in packages {
        let cwd = env::current_dir()?;
        env::set_current_dir(&pkg.root)?;
        let pkg_args = RunForPackageArgs::build(
            pkg,
            scarb_metadata,
            args,
            &cache_dir,
            &artifacts_dir_path,
            partitioning_config.clone(),
            predeployed_contracts.clone(),
            &ui,
        )?;
        env::set_current_dir(&cwd)?;
        all_package_args.push(pkg_args);
    }

    for pkg_args in all_package_args {
        let cwd = env::current_dir()?;
        env::set_current_dir(&pkg_args.package_root)?;

        let result = run_for_package(
            pkg_args,
            &block_number_map,
            ui.clone(),
            &mut exit_first_channel,
        )
        .await?;

        let filtered = result.filtered();
        all_tests.extend(result.summaries());
        total_filtered_count = calculate_total_filtered_count(total_filtered_count, filtered);
        env::set_current_dir(&cwd)?;
    }

    let overall_summary = OverallSummaryMessage::new(&all_tests, total_filtered_count);
    let mut all_failed_tests: Vec<&AnyTestCaseSummary> = extract_failed_tests(&all_tests).collect();
    if deterministic_output {
        all_failed_tests.sort_by(|a, b| a.name().unwrap_or("").cmp(b.name().unwrap_or("")));
    }

    FailedTestsCache::new(&cache_dir).save_failed_tests(&all_failed_tests)?;

    let url_to_block_number = block_number_map.get_url_to_latest_block_number();
    if !url_to_block_number.is_empty() {
        ui.println(&LatestBlocksNumbersMessage::new(url_to_block_number));
    }

    ui.println(&TestsFailureSummaryMessage::new(&all_failed_tests));

    // Print the overall summary only when testing multiple packages
    if packages_len > 1 {
        // Add newline to separate summary from previous output
        ui.print_blank_line();
        ui.println(&overall_summary);
    }

    match partitioning_config {
        PartitionConfig::Disabled => (),
        PartitionConfig::Enabled {
            partition,
            partition_map,
        } => {
            ui.print_blank_line();

            let included = partition_map.included_tests_count(partition.index());
            let total = partition_map.total_tests_count();
            ui.println(&PartitionFinishedMessage::new(partition, included, total));
        }
    }

    if args.exact {
        unset_forge_test_filter();
    }

    Ok(WorkspaceExecutionSummary { all_tests })
}

fn calculate_total_filtered_count(
    total_filtered_count: Option<usize>,
    filtered: Option<usize>,
) -> Option<usize> {
    // Calculate filtered test counts across packages. When using `--exact` flag,
    // `result.filtered_count` is None, so `total_filtered_count` becomes None too.
    match (total_filtered_count, filtered) {
        (Some(total), Some(f)) => Some(total + f),
        _ => None,
    }
}

fn get_partitioning_config(
    args: &TestArgs,
    ui: &UI,
    packages: &[PackageMetadata],
    artifacts_dir_path: &camino::Utf8Path,
) -> Result<PartitionConfig> {
    args.partition
        .map(|partition| {
            ui.print_blank_line();
            ui.println(&PartitionStartedMessage::new(partition));
            PartitionConfig::new(partition, packages, artifacts_dir_path)
        })
        .transpose()?
        .map_or_else(|| Ok(PartitionConfig::Disabled), Ok)
}

#[tracing::instrument(skip_all, level = "debug")]
fn extract_failed_tests(
    tests_summaries: &[TestTargetSummary],
) -> impl Iterator<Item = &AnyTestCaseSummary> {
    tests_summaries
        .iter()
        .flat_map(|summary| &summary.test_case_summaries)
        .filter(|s| s.is_failed())
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

fn should_load_predeployed_contracts_sierra(trace_args: &TraceArgs) -> bool {
    is_backtrace_enabled() || !trace_args.is_empty()
}
