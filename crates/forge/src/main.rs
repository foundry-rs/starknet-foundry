use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8Path;
use clap::{Parser, Subcommand, ValueEnum};
use forge::pretty_printing::print_warning;
use forge::scarb::config::ForgeConfig;
use forge::scarb::{
    build_contracts_with_scarb, build_test_artifacts_with_scarb, config_from_scarb_for_package,
};
use forge::shared_cache::{clean_cache, set_cached_failed_tests_names};
use forge::test_filter::TestsFilter;
use forge::{pretty_printing, run};
use forge_runner::test_case_summary::{AnyTestCaseSummary, TestCaseSummary};
use forge_runner::test_crate_summary::TestCrateSummary;
use forge_runner::{RunnerConfig, RunnerParams, CACHE_DIR};
use rand::{thread_rng, RngCore};
use scarb_api::{
    get_contracts_map, package_matches_version_requirement, target_dir_for_workspace, ScarbCommand,
};
use scarb_metadata::{Metadata, MetadataCommand, PackageMetadata};
use scarb_ui::args::PackagesFilter;

use forge::block_number_map::BlockNumberMap;
use semver::{Comparator, Op, Version, VersionReq};
use std::env;
use std::sync::Arc;
use std::thread::available_parallelism;
use tokio::runtime::Builder;

mod init;

const FUZZER_RUNS_DEFAULT: u32 = 256;

#[derive(Parser, Debug)]
#[command(version)]
#[clap(name = "snforge")]
struct Cli {
    #[command(subcommand)]
    subcommand: ForgeSubcommand,
}

#[derive(Subcommand, Debug)]
enum ForgeSubcommand {
    /// Run tests for a project in the current directory
    Test {
        #[command(flatten)]
        args: TestArgs,
    },
    /// Create a new directory with a Forge project
    Init {
        /// Name of a new project
        name: String,
    },
    /// Clean Forge cache directory
    CleanCache {},
}

#[derive(ValueEnum, Debug, Clone)]
enum ColorOption {
    Auto,
    Always,
    Never,
}

#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
struct TestArgs {
    /// Name used to filter tests
    test_filter: Option<String>,
    /// Use exact matches for `test_filter`
    #[arg(short, long)]
    exact: bool,

    /// Stop executing tests after the first failed test
    #[arg(short = 'x', long)]
    exit_first: bool,

    #[command(flatten)]
    packages_filter: PackagesFilter,

    /// Number of fuzzer runs
    #[arg(short = 'r', long, value_parser = validate_fuzzer_runs_value)]
    fuzzer_runs: Option<u32>,
    /// Seed for the fuzzer
    #[arg(short = 's', long)]
    fuzzer_seed: Option<u64>,

    /// Run only tests marked with `#[ignore]` attribute
    #[arg(long = "ignored")]
    only_ignored: bool,
    /// Run all tests regardless of `#[ignore]` attribute
    #[arg(long, conflicts_with = "only_ignored")]
    include_ignored: bool,

    /// Control when colored output is used
    #[arg(value_enum, long, default_value_t = ColorOption::Auto, value_name="WHEN")]
    color: ColorOption,

    /// Run tests that failed during the last run
    #[arg(long)]
    rerun_failed: bool,
}

fn validate_fuzzer_runs_value(val: &str) -> Result<u32> {
    let parsed_val: u32 = val
        .parse()
        .map_err(|_| anyhow!("Failed to parse {val} as u32"))?;
    if parsed_val < 3 {
        bail!("Number of fuzzer runs must be greater than or equal to 3")
    }
    Ok(parsed_val)
}

fn extract_failed_tests(tests_summaries: Vec<TestCrateSummary>) -> Vec<AnyTestCaseSummary> {
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
        .collect()
}

fn combine_configs(
    workspace_root: &Utf8Path,
    exit_first: bool,
    fuzzer_runs: Option<u32>,
    fuzzer_seed: Option<u64>,
    forge_config: &ForgeConfig,
) -> RunnerConfig {
    RunnerConfig::new(
        workspace_root.to_path_buf(),
        exit_first || forge_config.exit_first,
        fuzzer_runs
            .or(forge_config.fuzzer_runs)
            .unwrap_or(FUZZER_RUNS_DEFAULT),
        fuzzer_seed
            .or(forge_config.fuzzer_seed)
            .unwrap_or_else(|| thread_rng().next_u64()),
    )
}

fn snforge_std_version_requirement() -> VersionReq {
    let version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let comparator = Comparator {
        op: Op::Exact,
        major: version.major,
        minor: Some(version.minor),
        patch: Some(version.patch),
        pre: version.pre,
    };
    VersionReq {
        comparators: vec![comparator],
    }
}

fn warn_if_snforge_std_not_compatible(scarb_metadata: &Metadata) -> Result<()> {
    let snforge_std_version_requirement = snforge_std_version_requirement();
    if !package_matches_version_requirement(
        scarb_metadata,
        "snforge_std",
        &snforge_std_version_requirement,
    )? {
        print_warning(&anyhow!("Package snforge_std version does not meet the recommended version requirement {snforge_std_version_requirement}, it might result in unexpected behaviour"));
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn test_workspace(args: TestArgs) -> Result<bool> {
    match args.color {
        ColorOption::Always => env::set_var("CLICOLOR_FORCE", "1"),
        ColorOption::Never => env::set_var("CLICOLOR", "0"),
        ColorOption::Auto => (),
    }

    let scarb_metadata = MetadataCommand::new().inherit_stderr().exec()?;
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
            for package in &packages {
                env::set_current_dir(&package.root)?;

                let forge_config = config_from_scarb_for_package(&scarb_metadata, &package.id)?;
                let contracts = get_contracts_map(&scarb_metadata, &package.id).unwrap_or_default();

                let runner_config = Arc::new(combine_configs(
                    &workspace_root,
                    args.exit_first,
                    args.fuzzer_runs,
                    args.fuzzer_seed,
                    &forge_config,
                ));
                let runner_params = Arc::new(RunnerParams::new(contracts, env::vars().collect()));

                let tests_file_summaries = run(
                    &package.name,
                    &snforge_target_dir_path,
                    &TestsFilter::from_flags(
                        args.test_filter.clone(),
                        args.exact,
                        args.only_ignored,
                        args.include_ignored,
                        args.rerun_failed,
                        workspace_root.join(CACHE_DIR),
                    ),
                    runner_config,
                    runner_params,
                    &forge_config.fork,
                    &mut block_number_map,
                )
                .await?;

                let mut failed_tests = extract_failed_tests(tests_file_summaries);
                all_failed_tests.append(&mut failed_tests);
            }
            set_cached_failed_tests_names(&all_failed_tests, &workspace_root.join(CACHE_DIR))?;
            pretty_printing::print_latest_blocks_numbers(
                block_number_map.get_url_to_latest_block_number(),
            );

            Ok::<_, anyhow::Error>(all_failed_tests)
        })
    })??;

    pretty_printing::print_failures(&all_failed_tests);

    Ok(all_failed_tests.is_empty())
}

#[allow(clippy::too_many_lines)]
fn main_execution() -> Result<bool> {
    let cli = Cli::parse();

    ScarbCommand::new().ensure_available()?;

    match cli.subcommand {
        ForgeSubcommand::Init { name } => {
            init::run(name.as_str())?;
            Ok(true)
        }
        ForgeSubcommand::CleanCache {} => {
            clean_cache()?;
            Ok(true)
        }
        ForgeSubcommand::Test { args } => test_workspace(args),
    }
}

fn main() {
    match main_execution() {
        Ok(true) => std::process::exit(0),
        Ok(false) => std::process::exit(1),
        Err(error) => {
            pretty_printing::print_error_message(&error);
            std::process::exit(2);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;

    #[test]
    fn fuzzer_default_seed() {
        let workspace_root: Utf8PathBuf = Default::default();
        let config = combine_configs(&workspace_root, false, None, None, &Default::default());
        let config2 = combine_configs(&workspace_root, false, None, None, &Default::default());

        assert_ne!(config.fuzzer_seed, 0);
        assert_ne!(config2.fuzzer_seed, 0);
        assert_ne!(config.fuzzer_seed, config2.fuzzer_seed);
    }

    #[test]
    fn runner_config_default_arguments() {
        let workspace_root: Utf8PathBuf = Default::default();
        let config = combine_configs(&workspace_root, false, None, None, &Default::default());
        assert_eq!(
            config,
            RunnerConfig::new(
                workspace_root,
                false,
                FUZZER_RUNS_DEFAULT,
                config.fuzzer_seed,
            )
        );
    }

    #[test]
    fn runner_config_just_scarb_arguments() {
        let config_from_scarb = ForgeConfig {
            exit_first: true,
            fork: vec![],
            fuzzer_runs: Some(1234),
            fuzzer_seed: Some(500),
        };
        let workspace_root: Utf8PathBuf = Default::default();

        let config = combine_configs(&workspace_root, false, None, None, &config_from_scarb);
        assert_eq!(config, RunnerConfig::new(workspace_root, true, 1234, 500));
    }

    #[test]
    fn runner_config_argument_precedence() {
        let workspace_root: Utf8PathBuf = Default::default();

        let config_from_scarb = ForgeConfig {
            exit_first: false,
            fork: vec![],
            fuzzer_runs: Some(1234),
            fuzzer_seed: Some(1000),
        };
        let config = combine_configs(
            &workspace_root,
            true,
            Some(100),
            Some(32),
            &config_from_scarb,
        );

        assert_eq!(config, RunnerConfig::new(workspace_root, true, 100, 32,));
    }
}
