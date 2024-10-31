use clap::{Args, ValueEnum};
use starknet::core::types::Felt;

#[derive(ValueEnum, Debug, Clone)]
pub enum DeployVersion {
    V1,
    V3,
}

#[derive(Args)]
pub struct DeployArgs {
    // A package to deploy a contract from
    #[clap(long)]
    pub package: Option<String>,

    /// Calldata for the contract constructor
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
    pub constructor_calldata: Vec<Felt>,

    /// Salt for the address
    #[clap(short, long)]
    pub salt: Option<Felt>,

    /// If true, salt will be modified with an account address
    #[clap(long)]
    pub unique: bool,

    /// Nonce of the transaction. If not provided, nonce will be set automatically
    #[clap(short, long)]
    pub nonce: Option<Felt>,

    /// Version of the deployment
    #[clap(short, long)]
    pub version: Option<DeployVersion>,
}
