use crate::starknet_commands::script::init::Init;
use clap::{Args, Subcommand};

pub mod init;
pub mod run;

#[derive(Args)]
#[command(about = "Execute a deployment script")]
pub struct Script {
    /// Module name that contains the `main` function, which will be executed
    pub script_module_name: Option<String>,

    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init(Init),
}
