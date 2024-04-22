use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use configuration::load_package_config;
use forge::scarb::config::ForgeConfigFromScarb;
use forge::scarb::{
    build_contracts_with_scarb, build_test_artifacts_with_scarb, get_test_artifacts_path,
    load_test_artifacts,
};
use forge::shared_cache::{clean_cache, set_cached_failed_tests_names};
use forge::test_filter::TestsFilter;
use forge::{pretty_printing, run};
use forge_runner::test_case_summary::{AnyTestCaseSummary, TestCaseSummary};
use forge_runner::test_crate_summary::TestCrateSummary;
use forge_runner::CACHE_DIR;
use rand::{thread_rng, RngCore};
use scarb_api::{
    get_contracts_artifacts_and_source_sierra_paths,
    metadata::{Metadata, MetadataCommandExt, PackageMetadata},
    package_matches_version_requirement, target_dir_for_workspace, ScarbCommand,
};
use scarb_ui::args::PackagesFilter;

use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use forge::block_number_map::BlockNumberMap;
use forge_runner::forge_config::{
    is_vm_trace_needed, ExecutionDataToSave, ForgeConfig, OutputConfig, TestRunnerConfig,
};
use semver::{Comparator, Op, Version, VersionReq};
use shared::print::print_as_warning;
use std::env;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::thread::available_parallelism;
use tokio::runtime::Builder;
use universal_sierra_compiler_api::UniversalSierraCompilerCommand;

mod init;

fn default_fuzzer_runs() -> NonZeroU32 {
    NonZeroU32::new(256).unwrap()
}

#[derive(Parser, Debug)]
#[command(
    version,
    help_template = "\
{name} {version}
{author-with-newline}{about-with-newline}
Use -h for short descriptions and --help for more details.

{before-help}{usage-heading} {usage}

{all-args}{after-help}
",
    after_help = "Read the docs: https://foundry-rs.github.io/starknet-foundry/",
    after_long_help = "\
Read the docs:
- Starknet Foundry Book: https://foundry-rs.github.io/starknet-foundry/
- Cairo Book: https://book.cairo-lang.org/
- Starknet Book: https://book.starknet.io/
- Starknet Documentation: https://docs.starknet.io/
- Scarb Documentation: https://docs.swmansion.com/scarb/docs.html

Join the community:
- Follow core developers on X: https://twitter.com/swmansionxyz
- Get support via Telegram: https://t.me/starknet_foundry_support
- Or discord: https://discord.gg/KZWaFtPZJf
- Or join our general chat (Telegram): https://t.me/starknet_foundry

Report bugs: https://github.com/foundry-rs/starknet-foundry/issues/new/choose\
"
)]
#[command(about = "snforge - a testing tool for Starknet contracts", long_about = None)]
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
    #[arg(short = 'r', long)]
    fuzzer_runs: Option<NonZeroU32>,
    /// Seed for the fuzzer
    #[arg(short = 's', long)]
    fuzzer_seed: Option<u64>,

    /// Run only tests marked with `#[ignore]` attribute
    #[arg(long = "ignored")]
    only_ignored: bool,
    /// Run all tests regardless of `#[ignore]` attribute
    #[arg(long, conflicts_with = "only_ignored")]
    include_ignored: bool,

    /// Display more detailed info about used resources
    #[arg(long)]
    detailed_resources: bool,

    /// Control when colored output is used
    #[arg(value_enum, long, default_value_t = ColorOption::Auto, value_name="WHEN")]
    color: ColorOption,

    /// Run tests that failed during the last run
    #[arg(long)]
    rerun_failed: bool,

    /// Save execution traces of all test which have passed and are not fuzz tests
    #[arg(long)]
    save_trace_data: bool,

    /// Build profiles of all test which have passed and are not fuzz tests using the cairo-profiler
    #[arg(long)]
    build_profile: bool,

    /// Number of maximum steps during a single test. For fuzz tests this value is applied to each subtest separately.
    #[arg(long)]
    max_n_steps: Option<u32>,
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

#[allow(clippy::too_many_arguments)]
#[allow(clippy::fn_params_excessive_bools)]
fn combine_configs(
    exit_first: bool,
    fuzzer_runs: Option<NonZeroU32>,
    fuzzer_seed: Option<u64>,
    detailed_resources: bool,
    save_trace_data: bool,
    build_profile: bool,
    max_n_steps: Option<u32>,
    contracts_data: ContractsData,
    cache_dir: Utf8PathBuf,
    test_artifacts_path: Utf8PathBuf,
    forge_config_from_scarb: &ForgeConfigFromScarb,
) -> ForgeConfig {
    let execution_data_to_save = ExecutionDataToSave::from_flags(
        save_trace_data || forge_config_from_scarb.save_trace_data,
        build_profile || forge_config_from_scarb.build_profile,
    );

    ForgeConfig {
        test_runner_config: Arc::new(TestRunnerConfig {
            exit_first: exit_first || forge_config_from_scarb.exit_first,
            fuzzer_runs: fuzzer_runs
                .or(forge_config_from_scarb.fuzzer_runs)
                .unwrap_or(default_fuzzer_runs()),
            fuzzer_seed: fuzzer_seed
                .or(forge_config_from_scarb.fuzzer_seed)
                .unwrap_or_else(|| thread_rng().next_u64()),
            max_n_steps: max_n_steps.or(forge_config_from_scarb.max_n_steps),
            is_vm_trace_needed: is_vm_trace_needed(execution_data_to_save),
            cache_dir,
            contracts_data,
            environment_variables: env::vars().collect(),
            test_artifacts_path,
        }),
        output_config: Arc::new(OutputConfig {
            detailed_resources: detailed_resources || forge_config_from_scarb.detailed_resources,
            execution_data_to_save,
        }),
    }
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
        print_as_warning(&anyhow!("Package snforge_std version does not meet the recommended version requirement {snforge_std_version_requirement}, it might result in unexpected behaviour"));
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
                    test_artifacts_path,
                    &forge_config_from_scarb,
                ));

                let test_filter = TestsFilter::from_flags(
                    args.test_filter.clone(),
                    args.exact,
                    args.only_ignored,
                    args.include_ignored,
                    args.rerun_failed,
                    workspace_root.join(CACHE_DIR),
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

            set_cached_failed_tests_names(&all_failed_tests, &cache_dir)?;
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
    UniversalSierraCompilerCommand::ensure_available()?;

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

    #[test]
    fn fuzzer_default_seed() {
        let config = combine_configs(
            false,
            None,
            None,
            false,
            false,
            false,
            None,
            Default::default(),
            Default::default(),
            Default::default(),
            &Default::default(),
        );
        let config2 = combine_configs(
            false,
            None,
            None,
            false,
            false,
            false,
            None,
            Default::default(),
            Default::default(),
            Default::default(),
            &Default::default(),
        );

        assert_ne!(config.test_runner_config.fuzzer_seed, 0);
        assert_ne!(config2.test_runner_config.fuzzer_seed, 0);
        assert_ne!(
            config.test_runner_config.fuzzer_seed,
            config2.test_runner_config.fuzzer_seed
        );
    }

    #[test]
    fn runner_config_default_arguments() {
        let config = combine_configs(
            false,
            None,
            None,
            false,
            false,
            false,
            None,
            Default::default(),
            Default::default(),
            Default::default(),
            &Default::default(),
        );
        assert_eq!(
            config,
            ForgeConfig {
                test_runner_config: Arc::new(TestRunnerConfig {
                    exit_first: false,
                    fuzzer_runs: default_fuzzer_runs(),
                    fuzzer_seed: config.test_runner_config.fuzzer_seed,
                    max_n_steps: None,
                    is_vm_trace_needed: false,
                    cache_dir: Default::default(),
                    contracts_data: Default::default(),
                    environment_variables: config.test_runner_config.environment_variables.clone(),
                    test_artifacts_path: Default::default(),
                }),
                output_config: Arc::new(OutputConfig {
                    detailed_resources: false,
                    execution_data_to_save: ExecutionDataToSave::None
                }),
            }
        );
    }

    #[test]
    fn runner_config_just_scarb_arguments() {
        let config_from_scarb = ForgeConfigFromScarb {
            exit_first: true,
            fork: vec![],
            fuzzer_runs: Some(NonZeroU32::new(1234).unwrap()),
            fuzzer_seed: Some(500),
            detailed_resources: true,
            save_trace_data: true,
            build_profile: true,
            max_n_steps: Some(1_000_000),
        };

        let config = combine_configs(
            false,
            None,
            None,
            false,
            false,
            false,
            None,
            Default::default(),
            Default::default(),
            Default::default(),
            &config_from_scarb,
        );
        assert_eq!(
            config,
            ForgeConfig {
                test_runner_config: Arc::new(TestRunnerConfig {
                    exit_first: true,
                    fuzzer_runs: NonZeroU32::new(1234).unwrap(),
                    fuzzer_seed: 500,
                    max_n_steps: Some(1_000_000),
                    is_vm_trace_needed: true,
                    cache_dir: Default::default(),
                    contracts_data: Default::default(),
                    environment_variables: config.test_runner_config.environment_variables.clone(),
                    test_artifacts_path: Default::default(),
                }),
                output_config: Arc::new(OutputConfig {
                    detailed_resources: true,
                    execution_data_to_save: ExecutionDataToSave::TraceAndProfile
                }),
            }
        );
    }

    #[test]
    fn runner_config_argument_precedence() {
        let config_from_scarb = ForgeConfigFromScarb {
            exit_first: false,
            fork: vec![],
            fuzzer_runs: Some(NonZeroU32::new(1234).unwrap()),
            fuzzer_seed: Some(1000),
            detailed_resources: false,
            save_trace_data: false,
            build_profile: false,
            max_n_steps: Some(1234),
        };
        let config = combine_configs(
            true,
            Some(NonZeroU32::new(100).unwrap()),
            Some(32),
            true,
            true,
            true,
            Some(1_000_000),
            Default::default(),
            Default::default(),
            Default::default(),
            &config_from_scarb,
        );

        assert_eq!(
            config,
            ForgeConfig {
                test_runner_config: Arc::new(TestRunnerConfig {
                    exit_first: true,
                    fuzzer_runs: NonZeroU32::new(100).unwrap(),
                    fuzzer_seed: 32,
                    max_n_steps: Some(1_000_000),
                    is_vm_trace_needed: true,
                    cache_dir: Default::default(),
                    contracts_data: Default::default(),
                    environment_variables: config.test_runner_config.environment_variables.clone(),
                    test_artifacts_path: Default::default(),
                }),
                output_config: Arc::new(OutputConfig {
                    detailed_resources: true,
                    execution_data_to_save: ExecutionDataToSave::TraceAndProfile
                }),
            }
        );
    }
}
