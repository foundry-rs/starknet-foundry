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
    /// List aliases from the current effective config
    List,
}

#[expect(clippy::needless_pass_by_value)]
pub fn alias(alias: Alias, config: &CastConfig, ui: &UI) -> Result<ExitCode> {
    match alias.command {
        Commands::List => list::list(config, ui),
    }
}
