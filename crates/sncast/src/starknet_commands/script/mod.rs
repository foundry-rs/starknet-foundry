use crate::starknet_commands::script::init::Init;
use crate::starknet_commands::script::run::Run;
use clap::{Args, Subcommand};

pub mod init;
pub mod run;

#[derive(Args)]
pub struct Script {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init(Init),
    Run(Run),
}
