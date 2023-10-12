use clap::Args;
use starknet::core::types::FieldElement;
use cast::helpers::response_structs::VerifyResponse;

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
    #[clap(short = 'v', long = 'verifier')]
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
    Ok(VerifyResponse { verification_status, errors })
}