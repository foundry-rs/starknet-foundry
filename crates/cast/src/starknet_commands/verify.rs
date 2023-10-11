use clap::Args;
use starknet::core::types::contract_address;


#[derive(Args)]
#[command(about = "Verify a contract through a block exporer")]
pub struct Deploy {
    /// Address of a contract to be verified
    #[clap(short = 'a', long)]
    pub contract_address: contract_address,
}