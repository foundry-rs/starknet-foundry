use camino::Utf8PathBuf;
use clap::Args;
use starknet::core::types::FieldElement;

#[derive(Args, Debug)]
#[command(about = "Deploy an account to the Starknet")]
pub struct Deploy {}
