use anyhow::{Result, anyhow, bail};
use camino::Utf8PathBuf;
use clap::{Args, ValueEnum};
use promptly::prompt;
use scarb_api::StarknetContractArtifacts;
use sncast::{Network, response::structs::VerifyResponse};
use starknet_types_core::felt::Felt;
use std::{collections::HashMap, fmt};
pub mod explorer;
pub mod walnut;
use explorer::VerificationInterface;
use walnut::WalnutVerificationInterface;

#[derive(Args)]
#[command(about = "Verify a contract through a block explorer")]
pub struct Verify {
    /// Address of a contract to be verified
    #[clap(short = 'd', long)]
    pub contract_address: Felt,

    /// Name of the contract that is being verified
    #[clap(short, long)]
    pub contract_name: String,

    /// Block explorer to use for the verification
    #[clap(short, long, value_enum, default_value_t = Verifier::Walnut)]
    pub verifier: Verifier,

    /// The network on which block explorer will do the verification
    #[clap(short, long, value_enum)]
    pub network: Network,

    /// Assume "yes" as answer to confirmation prompt and run non-interactively
    #[clap(long, default_value = "false")]
    pub confirm_verification: bool,

    /// Specifies scarb package to be used
    #[clap(long)]
    pub package: Option<String>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Verifier {
    Walnut,
}

impl fmt::Display for Verifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Verifier::Walnut => write!(f, "walnut"),
        }
    }
}

pub async fn verify(
    contract_address: Felt,
    contract_name: String,
    verifier: Verifier,
    network: Network,
    confirm_verification: bool,
    manifest_path: &Utf8PathBuf,
    artifacts: &HashMap<String, StarknetContractArtifacts>,
) -> Result<VerifyResponse> {
    // Let's ask confirmation
    if !confirm_verification {
        let prompt_text = format!(
            "\n\tYou are about to submit the entire workspace code to the third-party verifier at {verifier}.\n\n\tImportant: Make sure your project does not include sensitive information like private keys. The snfoundry.toml file will be uploaded. Keep the keystore outside the project to prevent it from being uploaded.\n\n\tAre you sure you want to proceed? (Y/n)"
        );
        let input: String = prompt(prompt_text)?;
        if !input.starts_with('Y') {
            bail!("Verification aborted");
        }
    }

    if !artifacts.contains_key(&contract_name) {
        return Err(anyhow!("Contract named '{contract_name}' was not found"));
    }

    // Build JSON Payload for the verification request
    // get the parent dir of the manifest path
    let workspace_dir = manifest_path
        .parent()
        .ok_or(anyhow!("Failed to obtain workspace dir"))?;

    match verifier {
        Verifier::Walnut => {
            let walnut = WalnutVerificationInterface::new(network, workspace_dir.to_path_buf());
            walnut.verify(contract_address, contract_name).await
        }
    }
}
