use anyhow::Result;
use clap::{Args, Command};
use clap_complete::{Generator, Shell};
use foundry_ui::{Message, UI, components::warning::WarningMessage};
use std::io;

#[derive(Args, Debug)]
pub struct Completions {
    pub shell: Option<Shell>,
}

pub fn generate_completions_to_stdout<G: Generator>(shell: G, cmd: &mut Command) {
    clap_complete::generate(shell, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

pub fn generate_completions(shell: Option<Shell>, cmd: &mut Command, ui: &UI) -> Result<()> {
    let Some(shell) = shell.or_else(Shell::from_env) else {
        anyhow::bail!("Unsupported shell")
    };

    generate_completions_to_stdout(shell, cmd);

    // TODO(#3560): Remove this warning when the `completion` alias is removed
    if std::env::args().nth(1).as_deref() == Some("completion") {
        let message = &WarningMessage::new(
            "Command `completion` is deprecated and will be removed in the future. Please use `completions` instead.",
        );

        ui.println(&format!("# {}", message.text()));
    }

    Ok(())
}
