use anyhow::Result;
use clap::{Args, Command};
use clap_complete::{Generator, Shell};
use std::io;

#[derive(Args, Debug)]
pub struct Completion {
    pub shell: Option<Shell>,
}

pub fn generate_completions_to_stdout<G: Generator>(shell: G, cmd: &mut Command) {
    clap_complete::generate(shell, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

pub fn generate_completions(shell: Option<Shell>, cmd: &mut Command) -> Result<()> {
    if let Some(shell) = shell {
        generate_completions_to_stdout(shell, cmd);
    } else if let Some(shell) = Shell::from_env() {
        generate_completions_to_stdout(shell, cmd);
    } else {
        anyhow::bail!("Unsupported shell");
    }
    Ok(())
}
