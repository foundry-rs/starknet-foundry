use crate::starknet_commands::script::init::Init;
use clap::{Args, Subcommand};

pub mod init;
pub mod run;

#[derive(Args)]
pub struct Script {
    /// Module name that contains the `main` function, which will be executed
    pub module_name: Option<String>,

    /// Specifies scarb package to be used
    #[clap(long)]
    pub package: Option<String>,

    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init(Init),
}
