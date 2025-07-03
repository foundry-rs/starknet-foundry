use anyhow::{Result, anyhow, bail};
use camino::Utf8PathBuf;
use clap::{ArgGroup, Args, ValueEnum};
use promptly::prompt;
use scarb_api::StarknetContractArtifacts;
use sncast::{Network, response::verify::VerifyResponse};
use starknet_types_core::felt::Felt;
use std::{collections::HashMap, fmt};

pub mod explorer;
pub mod walnut;

use explorer::ContractIdentifier;
use explorer::VerificationInterface;
use walnut::WalnutVerificationInterface;

#[derive(Args)]
#[command(about = "Verify a contract through a block explorer")]
#[command(group(
    ArgGroup::new("contract_identifier")
        .required(true)
        .args(&["class_hash", "contract_address"])
))]
pub struct Verify {
    /// Class hash of a contract to be verified
    #[arg(short = 'g', long)]
    pub class_hash: Option<Felt>,

    /// Address of a contract to be verified
    #[arg(short = 'd', long)]
    pub contract_address: Option<Felt>,

    /// Name of the contract that is being verified
    #[arg(short, long)]
    pub contract_name: String,

    /// Block explorer to use for the verification
    #[arg(short, long, value_enum, default_value_t = Verifier::Walnut)]
    pub verifier: Verifier,

    /// The network on which block explorer will do the verification
    #[arg(short, long, value_enum)]
    pub network: Network,

    /// Assume "yes" as answer to confirmation prompt and run non-interactively
    #[arg(long, default_value = "false")]
    pub confirm_verification: bool,

    /// Specifies scarb package to be used
    #[arg(long)]
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
    verify: Verify,
    manifest_path: &Utf8PathBuf,
    artifacts: &HashMap<String, StarknetContractArtifacts>,
) -> Result<VerifyResponse> {
    let verifier = verify.verifier;

    // Let's ask confirmation
    if !verify.confirm_verification {
        let prompt_text = format!(
            "\n\tYou are about to submit the entire workspace code to the third-party verifier at {verifier}.\n\n\tImportant: Make sure your project does not include sensitive information like private keys. The snfoundry.toml file will be uploaded. Keep the keystore outside the project to prevent it from being uploaded.\n\n\tAre you sure you want to proceed? (Y/n)"
        );
        let input: String = prompt(prompt_text)?;

        if !input.starts_with('Y') {
            bail!("Verification aborted");
        }
    }

    let contract_name = verify.contract_name;
    if !artifacts.contains_key(&contract_name) {
        return Err(anyhow!("Contract named '{contract_name}' was not found"));
    }

    // Build JSON Payload for the verification request
    // get the parent dir of the manifest path
    let workspace_dir = manifest_path
        .parent()
        .ok_or(anyhow!("Failed to obtain workspace dir"))?;

    let contract_identifier = match (verify.class_hash, verify.contract_address) {
        (Some(class_hash), None) => ContractIdentifier::ClassHash {
            class_hash: class_hash.to_fixed_hex_string(),
        },
        (None, Some(contract_address)) => ContractIdentifier::Address {
            contract_address: contract_address.to_fixed_hex_string(),
        },

        _ => {
            unreachable!("Exactly one of class_hash or contract_address must be provided.");
        }
    };

    match verifier {
        Verifier::Walnut => {
            let walnut =
                WalnutVerificationInterface::new(verify.network, workspace_dir.to_path_buf());
            walnut.verify(contract_identifier, contract_name).await
        }
    }
}
