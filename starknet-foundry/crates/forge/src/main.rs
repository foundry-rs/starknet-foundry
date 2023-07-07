use anyhow::{Context, Result};
use clap::Parser;
use forge::run;
use forge::{pretty_printing, RunnerConfig};
use scarb_metadata::MetadataCommand;

use std::process::Command;

#[derive(Parser, Debug)]
struct Args {
    /// Name used to filter tests
    test_name: Option<String>,
    /// Use exact matches for `test_filter`
    #[arg(short, long)]
    exact: bool,

    #[arg(short = 'x', long)]
    exit_first: bool,
}

fn main_execution() -> Result<()> {
    let args = Args::parse();

    let scarb_metadata = MetadataCommand::new().inherit_stderr().exec()?;
    let _ = Command::new("scarb")
        .current_dir(std::env::current_dir().context("Failed to get current directory")?)
        .arg("build")
        .output()
        .context("Failed to build contracts with Scarb")?;

    for package in &scarb_metadata.workspace.members {
        let protostar_config = forge::protostar_config_for_package(&scarb_metadata, package)?;
        let (base_path, dependencies) = forge::dependencies_for_package(&scarb_metadata, package)?;
        let runner_config = RunnerConfig::new(
            args.test_name.clone(),
            args.exact,
            args.exit_first,
            &protostar_config,
        );

        run(&base_path, Some(dependencies.clone()), &runner_config)?;
    }
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
