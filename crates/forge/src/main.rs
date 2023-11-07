use anyhow::{anyhow, bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Parser, Subcommand, ValueEnum};
use forge::scarb::config_from_scarb_for_package;
use include_dir::{include_dir, Dir};
use scarb_artifacts::{
    corelib_for_package, dependencies_for_package, get_contracts_map, name_for_package,
    paths_for_package,
};
use scarb_metadata::{Metadata, MetadataCommand, PackageMetadata};
use scarb_ui::args::PackagesFilter;
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fs};
use tempfile::{tempdir, TempDir};
use tokio::runtime::Builder;

use forge::{pretty_printing, RunnerConfig, RunnerParams, CACHE_DIR, FUZZER_RUNS_DEFAULT};

use forge::{run, TestCrateSummary};

use forge::scarb::config::ForgeConfig;
use forge::test_case_summary::TestCaseSummary;
use forge::test_filter::TestsFilter;
use rand::{thread_rng, RngCore};
use std::process::{Command, Stdio};
use std::thread::available_parallelism;

mod init;

static PREDEPLOYED_CONTRACTS: Dir = include_dir!("crates/cheatnet/predeployed-contracts");

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

fn clean_cache() -> Result<()> {
    let scarb_metadata = MetadataCommand::new().inherit_stderr().exec()?;
    let workspace_root = scarb_metadata.workspace.root.clone();
    let cache_dir = workspace_root.join(CACHE_DIR);
    if cache_dir.exists() {
        fs::remove_dir_all(cache_dir)?;
    }
    Ok(())
}

fn load_predeployed_contracts() -> Result<TempDir> {
    let tmp_dir = tempdir()?;
    PREDEPLOYED_CONTRACTS
        .extract(&tmp_dir)
        .context("Failed to copy corelib to temporary directory")?;
    Ok(tmp_dir)
}

fn extract_failed_tests(tests_summaries: Vec<TestCrateSummary>) -> Vec<TestCaseSummary> {
    tests_summaries
        .into_iter()
        .flat_map(|test_file_summary| test_file_summary.test_case_summaries)
        .filter(|test_case_summary| matches!(test_case_summary, TestCaseSummary::Failed { .. }))
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
        forge_config.fork.clone(),
        fuzzer_runs
            .or(forge_config.fuzzer_runs)
            .unwrap_or(FUZZER_RUNS_DEFAULT),
        fuzzer_seed
            .or(forge_config.fuzzer_seed)
            .unwrap_or_else(|| thread_rng().next_u64()),
    )
}

fn test_workspace(args: TestArgs) -> Result<bool> {
    match args.color {
        ColorOption::Always => env::set_var("CLICOLOR_FORCE", "1"),
        ColorOption::Never => env::set_var("CLICOLOR", "0"),
        ColorOption::Auto => (),
    }

    let predeployed_contracts_dir = load_predeployed_contracts()?;
    let predeployed_contracts_path: PathBuf = predeployed_contracts_dir.path().into();
    let predeployed_contracts = Utf8PathBuf::try_from(predeployed_contracts_path.clone())
        .context("Failed to convert path to predeployed contracts to Utf8PathBuf")?;

    let scarb_metadata = MetadataCommand::new().inherit_stderr().exec()?;
    let workspace_root = scarb_metadata.workspace.root.clone();

    let packages: Vec<PackageMetadata> = args
        .packages_filter
        .match_many(&scarb_metadata)
        .context("Failed to find any packages matching the specified filter")?;

    let filter = PackagesFilter::generate_for::<Metadata>(packages.iter());
    let build_output = Command::new("scarb")
        .arg("build")
        .env("SCARB_PACKAGES_FILTER", filter.to_env())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .context("Failed to build contracts with Scarb")?;
    if !build_output.status.success() {
        bail!("Scarb build did not succeed")
    }

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
            let mut all_failed_tests = vec![];
            for package in &packages {
                let forge_config = config_from_scarb_for_package(&scarb_metadata, &package.id)?;
                let (package_path, package_source_dir_path) =
                    paths_for_package(&scarb_metadata, &package.id)?;
                env::set_current_dir(package_path.clone())?;

                let package_name = Arc::new(name_for_package(&scarb_metadata, &package.id)?);
                let dependencies = dependencies_for_package(&scarb_metadata, &package.id)?;
                let corelib_path = corelib_for_package(&scarb_metadata, &package.id)?;

                let contracts = get_contracts_map(&scarb_metadata, &package.id).unwrap_or_default();

                let runner_config = Arc::new(combine_configs(
                    &workspace_root,
                    args.exit_first,
                    args.fuzzer_runs,
                    args.fuzzer_seed,
                    &forge_config,
                ));

                let runner_params = Arc::new(RunnerParams::new(
                    corelib_path,
                    contracts,
                    predeployed_contracts.clone(),
                    env::vars().collect(),
                    dependencies,
                ));

                let tests_file_summaries = run(
                    &package_path,
                    &package_name,
                    &package_source_dir_path,
                    &TestsFilter::from_flags(
                        args.test_filter.clone(),
                        args.exact,
                        args.only_ignored,
                        args.include_ignored,
                    ),
                    runner_config,
                    runner_params,
                )
                .await?;

                let mut failed_tests = extract_failed_tests(tests_file_summaries);
                all_failed_tests.append(&mut failed_tests);
            }
            Ok::<_, anyhow::Error>(all_failed_tests)
        })
    })??;

    // Explicitly close the temporary directories so we can handle the errors
    predeployed_contracts_dir.close().with_context(|| {
        anyhow!(
            "Failed to close temporary directory = {} with predeployed contracts. Predeployed contract files might have not been released from filesystem",
            predeployed_contracts_path.display()
        )
    })?;

    pretty_printing::print_failures(&all_failed_tests);

    Ok(all_failed_tests.is_empty())
}

#[allow(clippy::too_many_lines)]
fn main_execution() -> Result<bool> {
    let cli = Cli::parse();

    which::which("scarb")
        .context("Cannot find `scarb` binary in PATH. Make sure you have Scarb installed https://github.com/software-mansion/scarb")?;

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
            RunnerConfig {
                workspace_root,
                exit_first: false,
                fork_targets: vec![],
                fuzzer_runs: FUZZER_RUNS_DEFAULT,
                fuzzer_seed: config.fuzzer_seed,
            }
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
        assert_eq!(
            config,
            RunnerConfig {
                workspace_root,
                exit_first: true,
                fork_targets: vec![],
                fuzzer_runs: 1234,
                fuzzer_seed: 500,
            }
        );
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
        assert_eq!(
            config,
            RunnerConfig {
                workspace_root,
                exit_first: true,
                fork_targets: vec![],
                fuzzer_runs: 100,
                fuzzer_seed: 32,
            }
        );
    }
}
