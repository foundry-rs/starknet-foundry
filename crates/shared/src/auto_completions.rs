use anyhow::Result;
use clap::{Args, Command};
use clap_complete::{generate, Generator, Shell};
use std::io;

#[derive(Args, Debug)]
pub struct Completion {
    pub shell: Option<Shell>,
}

pub fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

pub fn generate_completions(shell: Option<Shell>, cmd: &mut Command) -> Result<()> {
    if let Some(shell) = shell {
        print_completions(shell, cmd);
    } else if let Some(shell) = Shell::from_env() {
        print_completions(shell, cmd);
    } else {
        anyhow::bail!("Unsupported shell");
    }
    Ok(())
}
