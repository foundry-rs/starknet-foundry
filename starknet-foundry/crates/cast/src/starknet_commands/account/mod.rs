use crate::starknet_commands::account::create::Create;
use crate::starknet_commands::account::deploy::Deploy;
use clap::{Args, Subcommand};

pub mod create;
pub mod deploy;

#[derive(Args)]
#[command(about = "Creates and deploys an account to the Starknet")]
pub struct Account {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Create(Create),
    Deploy(Deploy),
}
