use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use clap::Parser;
use include_dir::{include_dir, Dir};
use scarb_metadata::MetadataCommand;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

use forge::run;
use forge::{pretty_printing, RunnerConfig};

use forge::scarb::{get_contracts_map, try_get_starknet_artifacts_path};
use std::process::Command;

static CORELIB_PATH: Dir = include_dir!("../corelib/src");
static PREDEPLOYED_CONTRACTS: Dir = include_dir!("crates/cheatable-starknet/predeployed-contracts");

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Name used to filter tests
    test_name: Option<String>,
    /// Use exact matches for `test_filter`
    #[arg(short, long)]
    exact: bool,

    /// Stop test execution after first failed test
    #[arg(short = 'x', long)]
    exit_first: bool,
}

fn load_corelib() -> Result<TempDir> {
    let tmp_dir = tempdir()?;
    CORELIB_PATH
        .extract(&tmp_dir)
        .context("Failed to copy corelib to temporary directory")?;
    Ok(tmp_dir)
}

fn load_predeployed_contracts() -> Result<TempDir> {
    let tmp_dir = tempdir()?;
    PREDEPLOYED_CONTRACTS
        .extract(&tmp_dir)
        .context("Failed to copy corelib to temporary directory")?;
    Ok(tmp_dir)
}

fn main_execution() -> Result<()> {
    let args = Args::parse();

    // TODO #1997
    let corelib_dir = load_corelib()?;
    let corelib_path: PathBuf = corelib_dir.path().into();
    let corelib = Utf8PathBuf::try_from(corelib_path)
        .context("Failed to convert corelib path to Utf8PathBuf")?;

    let predeployed_contracts_dir = load_predeployed_contracts()?;
    let predeployed_contracts_path: PathBuf = predeployed_contracts_dir.path().into();
    let predeployed_contracts = Utf8PathBuf::try_from(predeployed_contracts_path.clone())
        .context("Failed to convert path to predeployed contracts to Utf8PathBuf")?;

    which::which("scarb")
        .context("Cannot find `scarb` binary in PATH. Make sure you have Scarb installed https://github.com/software-mansion/scarb")?;

    let scarb_metadata = MetadataCommand::new().inherit_stderr().exec()?;
    Command::new("scarb")
        .current_dir(std::env::current_dir().context("Failed to get current directory")?)
        .arg("build")
        .output()
        .context("Failed to build contracts with Scarb")?;

    for package in &scarb_metadata.workspace.members {
        let forge_config = forge::scarb::config_from_scarb_for_package(&scarb_metadata, package)?;

        let (package_path, lib_path, _corelib_path, dependencies, target_name) =
            forge::scarb::dependencies_for_package(&scarb_metadata, package)?;
        let runner_config = RunnerConfig::new(
            args.test_name.clone(),
            args.exact,
            args.exit_first,
            &forge_config,
        );

        let contracts_path = try_get_starknet_artifacts_path(&package_path, &target_name)?;
        let contracts = contracts_path
            .map(|path| get_contracts_map(&path))
            .transpose()?
            .unwrap_or_default();

        run(
            &package_path,
            &lib_path,
            &Some(dependencies.clone()),
            &runner_config,
            Some(&corelib),
            &contracts,
            &predeployed_contracts,
        )?;
    }

    // Explicitly close the temporary directories so we can handle the errors
    predeployed_contracts_dir.close().with_context(|| {
        anyhow!(
            "Failed to close temporary directory = {} with predeployed contracts. Predeployed contract files might have not been released from filesystem",
            predeployed_contracts_path.display()
        )
    })?;

    Ok(())
}

fn main() {
    match main_execution() {
        Ok(()) => std::process::exit(0),
        Err(error) => {
            pretty_printing::print_error_message(&error);
            std::process::exit(1);
        }
    };
}
