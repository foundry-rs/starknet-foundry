use anyhow::{anyhow, Result};
use clap::Args;
use starknet::core::types::FieldElement;
use cast::helpers::response_structs::VerifyResponse;
use cast::helpers::response_structs::VerificationStatus;

#[derive(Args)]
#[command(about = "Verify a contract through a block exporer")]
pub struct Verify {
    /// Address of a contract to be verified
    #[clap(short = 'a', long)]
    pub contract_address: FieldElement,

    /// Name of the contract that is being verified
    #[clap(short, long)]
    pub contract_name: String, 

    /// Block explorer to use for the verification
    #[clap(short = 'v', long = "verifier")]
    pub verifier: String,

    /// The network on which block explorer will do the verification
    #[clap(short = 'n', long = "network")]
    pub network: String,
}

pub async fn verify(
    contract_address: FieldElement,
    contract_name: String,
    verifier: String,
    network: String,
) -> Result<VerifyResponse> {
    let verification_status = VerificationStatus::OK;
    let errors = None;
    println!("Contract address: {}", contract_address);
    println!("Contract name: {}", contract_name);
    println!("Verifier: {}", verifier);
    println!("Network: {}", network);
    match verification_status {
        VerificationStatus::OK => {
            Ok(VerifyResponse {
                verification_status,
                errors
            })
        },
        VerificationStatus::Error => {
            Err(anyhow!("Unknown RPC error"))
        }
    }
    // Main core logic of verification starts from here
}