use clap::Args;
use sncast::helpers::{deploy::DeployArgs, fee::FeeToken};

#[derive(Args)]
#[command(about = "Deploy a contract on Starknet")]
pub struct DeclareDeploy {
    // Name of the contract to deploy
    #[clap(long)]
    pub contract_name: String,

    #[clap(flatten)]
    pub deploy_args: DeployArgs,

    /// Token that transaction fee will be paid in
    #[clap(long)]
    pub fee_token: FeeToken,
}
