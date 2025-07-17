use anyhow::Result;
use clap::{Args, Command};
use clap_complete::{Generator, Shell};
use std::io;

#[derive(Args, Debug)]
pub struct Completions {
    pub shell: Option<Shell>,
}

pub fn generate_completions_to_stdout<G: Generator>(shell: G, cmd: &mut Command) {
    clap_complete::generate(shell, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

pub fn generate_completions(shell: Option<Shell>, cmd: &mut Command) -> Result<()> {
    let Some(shell) = shell.or_else(Shell::from_env) else {
        anyhow::bail!("Unsupported shell")
    };

    generate_completions_to_stdout(shell, cmd);
    Ok(())
}
