use anyhow::Context;
use std::path::Path;
use std::str::from_utf8;

pub use command::*;

mod command;

#[must_use]
pub fn compile_sierra(sierra_file_name: &str, current_dir: Option<&Path>) -> String {
    let mut usc_command = UniversalSierraCompilerCommand::new();
    if let Some(dir) = current_dir {
        usc_command.current_dir(dir);
    }

    let usc_output = usc_command
        .inherit_stderr()
        .arg("--sierra-input-path")
        .arg(sierra_file_name)
        .command()
        .output()
        .context(
            "Error while compiling Sierra of the contract. \
            Make sure you have the latest universal-sierra-binary installed. \
            Contact us if it doesn't help",
        )
        .unwrap();

    let casm = from_utf8(&usc_output.stdout).unwrap();
    casm.to_string()
}
