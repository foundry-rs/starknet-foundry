use crate::compatibility_check::{create_version_parser, Requirement, RequirementsChecker};
use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use forge_runner::CACHE_DIR;
use run_tests::workspace::run_for_workspace;
use scarb_api::{metadata::MetadataCommandExt, ScarbCommand};
use scarb_ui::args::{FeaturesSpec, PackagesFilter};
use semver::Version;
use std::ffi::OsString;
use std::{fs, num::NonZeroU32, thread::available_parallelism};
use tokio::runtime::Builder;

pub mod block_number_map;
mod combine_configs;
mod compatibility_check;
mod init;
pub mod pretty_printing;
pub mod run_tests;
pub mod scarb;
mod shared_cache;
pub mod test_filter;
mod warn;

pub const CAIRO_EDITION: &str = "2024_07";

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
pub struct Cli {
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
pub struct TestArgs {
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

    /// Build profiles of all tests which have passed and are not fuzz tests using the cairo-profiler
    #[arg(long, conflicts_with = "coverage")]
    build_profile: bool,

    /// Generate a coverage report for the executed tests which have passed and are not fuzz tests using the cairo-coverage
    #[arg(long, conflicts_with = "build_profile")]
    coverage: bool,

    /// Number of maximum steps during a single test. For fuzz tests this value is applied to each subtest separately.
    #[arg(long)]
    max_n_steps: Option<u32>,

    /// Specify features to enable
    #[command(flatten)]
    pub features: FeaturesSpec,

    /// Build contracts separately in the scarb starknet contract target
    #[arg(long)]
    no_optimization: bool,

    /// Additional arguments for cairo-coverage or cairo-profiler
    #[clap(last = true)]
    additional_args: Vec<OsString>,
}

pub enum ExitStatus {
    Success,
    Failure,
}

pub fn main_execution() -> Result<ExitStatus> {
    let cli = Cli::parse();

    let mut requirements_checker = RequirementsChecker::new();
    requirements_checker.add_requirement(Requirement {
        name: "Rust".to_string(),
        command: "rustc --version".to_string(),
        version_parser: create_version_parser("Rust", r"rustc (?<version>[0-9]+.[0-9]+.[0-9]+)"),
        helper_text: "Follow instructions from https://www.rust-lang.org/tools/install".to_string(),
        minimal_version: Version::new(1, 81, 0),
    });
    requirements_checker.add_requirement(Requirement {
        name: "Scarb".to_string(),
        command: "scarb --version".to_string(),
        minimal_version: Version::new(2, 7, 0),
        helper_text: "Follow instructions from https://docs.swmansion.com/scarb/download.html"
            .to_string(),
        version_parser: create_version_parser("Scarb", r"scarb (?<version>[0-9]+.[0-9]+.[0-9]+)"),
    });
    requirements_checker.add_requirement(Requirement {
        name: "Universal Sierra Compiler".to_string(),
        command: "universal-sierra-compiler --version".to_string(),
        minimal_version: Version::new(2, 0, 0),
        helper_text: "Reinstall `snforge` using the same installation method or follow instructions from https://foundry-rs.github.io/starknet-foundry/getting-started/installation.html#universal-sierra-compiler-update".to_string(),
        version_parser: create_version_parser(
        "Universal Sierra Compiler",
        r"universal-sierra-compiler (?<version>[0-9]+.[0-9]+.[0-9]+)",
        ),
    });
    requirements_checker.validate()?;

    match cli.subcommand {
        ForgeSubcommand::Init { name } => {
            init::run(name.as_str())?;
            Ok(ExitStatus::Success)
        }
        ForgeSubcommand::CleanCache {} => {
            let scarb_metadata = ScarbCommand::metadata().inherit_stderr().run()?;
            let cache_dir = scarb_metadata.workspace.root.join(CACHE_DIR);

            if cache_dir.exists() {
                fs::remove_dir_all(&cache_dir)?;
            }

            Ok(ExitStatus::Success)
        }
        ForgeSubcommand::Test { args } => {
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

            rt.block_on(run_for_workspace(args))
        }
    }
}
