use clap::{Args, Subcommand};

use crate::starknet_commands::utils::serialize::Serialize;

pub mod serialize;

#[derive(Args)]
#[command(about = "Utility commands for Starknet")]
pub struct Utils {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Serialize(Serialize),
}
