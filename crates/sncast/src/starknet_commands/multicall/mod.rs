use clap::{Args, Subcommand};

pub mod new;
pub mod run;

use new::New;
use run::Run;

#[derive(Args)]
#[command(about = "Execute multiple calls at once", long_about = None)]
pub struct Multicall {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Run(Run),
    New(New),
}
