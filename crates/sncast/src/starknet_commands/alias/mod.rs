use crate::starknet_commands::alias::list::List;
use anyhow::Result;
use clap::{Args, Subcommand};
use sncast::helpers::configuration::CastConfig;
use sncast::response::ui::UI;
use std::process::ExitCode;

pub mod list;

#[derive(Args)]
#[command(about = "Manage aliases from snfoundry.toml")]
pub struct Alias {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    List(List),
}

#[allow(clippy::unnecessary_wraps)]
pub fn alias(alias: &Alias, config: &CastConfig, ui: &UI) -> Result<ExitCode> {
    match alias.command {
        Commands::List(_) => {
            ui.print_message("alias list", list::list(config));
            Ok(ExitCode::SUCCESS)
        }
    }
}
