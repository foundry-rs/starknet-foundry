use crate::starknet_commands::account::create::Create;
use crate::starknet_commands::account::deploy::Deploy;
use clap::{Args, Subcommand};

pub mod create;
pub mod deploy;

pub const OZ_CLASS_HASH: &str =
    "0x058d97f7d76e78f44905cc30cb65b91ea49a4b908a76703c54197bca90f81773";

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
