use camino::Utf8PathBuf;
use clap::{Args, Subcommand};

pub mod new;
pub mod run;

#[derive(Args)]
#[command(about = "Execute multiple calls at once", long_about = None)]
pub struct Multicall {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Run {
        /// path to the toml file with declared operations
        #[clap(short = 'p', long = "path")]
        path: Utf8PathBuf,
    },
    New {
        /// output path to the file where the template is going to be saved
        #[clap(short = 'p', long = "output-path")]
        output_path: Option<Utf8PathBuf>,

        /// if the file specified in output-path exists, this flag decides if it is going to be overwritten
        #[clap(short = 'o', long = "overwrite")]
        overwrite: Option<bool>,
    },
}
