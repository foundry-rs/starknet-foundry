use super::base::VerificationInterface;
use super::voyager::VoyagerVerificationInterface;
use super::walnut::WalnutVerificationInterface;
use anyhow::{anyhow, bail, Result};
use camino::Utf8PathBuf;
use clap::{Parser, ValueEnum};
use promptly::prompt;
use scarb_api::StarknetContractArtifacts;
use sncast::response::structs::VerifyResponse;
use sncast::Network;
use starknet::core::types::FieldElement;
use std::collections::HashMap;
use std::fmt;

#[derive(Parser)]
#[command(about = "Verify a contract through a block explorer")]
pub struct Verify {
    #[clap(short = 'a', long)]
    pub contract_address: Option<FieldElement>,

    #[clap(short = 'x', long)]
    pub class_hash: Option<FieldElement>,

    #[clap(short, long)]
    pub class_name: String,

    #[clap(short, long, value_enum, default_value_t = Verifier::Walnut)]
    pub verifier: Verifier,

    #[clap(short, long, value_enum)]
    pub network: Network,

    #[clap(long, default_value = "false")]
    pub confirm_verification: bool,

    #[clap(long)]
    pub package: Option<String>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Verifier {
    Walnut,
    Voyager,
}

impl fmt::Display for Verifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Verifier::Walnut => write!(f, "walnut"),
            Verifier::Voyager => write!(f, "voyager"),
        }
    }
}

// disable too many arguments clippy warning
#[allow(clippy::too_many_arguments)]
pub async fn verify(
    contract_address: Option<FieldElement>,
    class_hash: Option<FieldElement>,
    class_name: String,
    verifier: Verifier,
    network: Network,
    confirm_verification: bool,
    manifest_path: &Utf8PathBuf,
    artifacts: &HashMap<String, StarknetContractArtifacts>,
) -> Result<VerifyResponse> {
    // Let's ask confirmation
    if !confirm_verification {
        let prompt_text = format!(
            "You are about to submit the entire workspace's code to the third-party chosen verifier at {verifier}, and the code will be publicly available through {verifier}'s APIs. Are you sure? (Y/n)"
        );
        let input: String = prompt(prompt_text)?;

        if !input.starts_with('Y') {
            bail!("Verification aborted");
        }
    }

    if !artifacts.contains_key(&class_name) {
        return Err(anyhow!("Contract named '{class_name}' was not found"));
    }

    // ensure that either contract_address or class_hash is provided
    if contract_address.is_none() && class_hash.is_none() {
        bail!("Either contract_address or class_hash must be provided");
    }

    // Build JSON Payload for the verification request
    // get the parent dir of the manifest path
    let workspace_dir = manifest_path
        .parent()
        .ok_or(anyhow!("Failed to obtain workspace dir"))?;

    match verifier {
        Verifier::Walnut => {
            let walnut = WalnutVerificationInterface::new(network, workspace_dir.to_path_buf());
            walnut
                .verify(contract_address, class_hash, class_name)
                .await
        }
        Verifier::Voyager => {
            let voyager = VoyagerVerificationInterface::new(network, workspace_dir.to_path_buf());
            voyager
                .verify(contract_address, class_hash, class_name)
                .await
        }
    }
}
