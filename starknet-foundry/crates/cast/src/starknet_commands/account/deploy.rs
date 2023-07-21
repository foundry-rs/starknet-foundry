use camino::Utf8PathBuf;
use clap::Args;
use starknet::core::types::FieldElement;

#[derive(Args, Debug)]
#[command(about = "Deploy an account to the Starknet")]
pub struct Deploy {
    /// Path to the file where the account secrets are stored
    #[clap(short, long)]
    path: Utf8PathBuf,

    /// Name of the account to be deployed
    #[clap(short, long)]
    name: String,

    #[clap(short, long)]
    max_fee: Option<FieldElement>,
}
