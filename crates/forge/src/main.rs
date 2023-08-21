use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use clap::Parser;
use include_dir::{include_dir, Dir};
use scarb_metadata::MetadataCommand;
use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

use forge::run;
use forge::{pretty_printing, RunnerConfig};

use forge::scarb::{
    corelib_for_package, dependencies_for_package, get_contracts_map, name_for_package,
    paths_for_package, target_name_for_package, try_get_starknet_artifacts_path,
};
use std::process::Command;

static PREDEPLOYED_CONTRACTS: Dir = include_dir!("crates/cheatnet/predeployed-contracts");

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

fn load_predeployed_contracts() -> Result<TempDir> {
    let tmp_dir = tempdir()?;
    PREDEPLOYED_CONTRACTS
        .extract(&tmp_dir)
        .context("Failed to copy corelib to temporary directory")?;
    Ok(tmp_dir)
}

fn main_execution() -> Result<()> {
    let args = Args::parse();

    let predeployed_contracts_dir = load_predeployed_contracts()?;
    let predeployed_contracts_path: PathBuf = predeployed_contracts_dir.path().into();
    let predeployed_contracts = Utf8PathBuf::try_from(predeployed_contracts_path.clone())
        .context("Failed to convert path to predeployed contracts to Utf8PathBuf")?;

    which::which("scarb")
        .context("Cannot find `scarb` binary in PATH. Make sure you have Scarb installed https://github.com/software-mansion/scarb")?;

    let scarb_metadata = MetadataCommand::new().inherit_stderr().exec()?;

    let build_output = Command::new("scarb")
        .current_dir(std::env::current_dir().context("Failed to get current directory")?)
        .arg("build")
        .output()
        .context("Failed to build contracts with Scarb")?;
    if !build_output.status.success() {
        bail!(
            "Scarb build didn't succeed:\n\n{}",
            String::from_utf8_lossy(&build_output.stdout)
        )
    }
    let scarb_stdout = String::from_utf8_lossy(&build_output.stdout);

    if scarb_stdout.contains("warn:") {
        eprintln!("{}", scarb_stdout);
    }

    for package in &scarb_metadata.workspace.members {
        let forge_config = forge::scarb::config_from_scarb_for_package(&scarb_metadata, package)?;
        let (package_path, lib_path) = paths_for_package(&scarb_metadata, package)?;

        std::env::set_current_dir(package_path.clone())?;

        let package_name = name_for_package(&scarb_metadata, package)?;
        let dependencies = dependencies_for_package(&scarb_metadata, package)?;
        let target_name = target_name_for_package(&scarb_metadata, package)?;
        let corelib_path = corelib_for_package(&scarb_metadata, package)?;
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
            &package_name,
            &lib_path,
            &Some(dependencies.clone()),
            &runner_config,
            &corelib_path,
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
