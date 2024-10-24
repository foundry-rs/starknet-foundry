use anyhow::{anyhow, bail, Result};
use base::VerificationInterface;
use camino::Utf8PathBuf;
use clap::{Args, Parser, ValueEnum};
use promptly::prompt;
use scarb_api::StarknetContractArtifacts;
use sncast::response::structs::VerifyResponse;
use sncast::Network;
use starknet::core::types::Felt;
use std::collections::HashMap;
use std::fmt;
use voyager::VoyagerVerificationInterface;
use walnut::WalnutVerificationInterface;

pub mod base;
mod voyager;
mod walnut;

#[derive(Args, Debug, Clone)]
#[group(required = true, multiple = false)]
pub struct ContractAddressOrClassHashGroup {
    /// Contract address of the contract. Either this or class hash should be provided.
    #[clap(short = 'd', long)]
    pub contract_address: Option<Felt>,

    /// Class hash of the contract. Either this or contract address should be provided.
    #[clap(short = 'x', long)]
    pub class_hash: Option<Felt>,
}

#[derive(Parser)]
#[command(about = "Verify a contract through a block explorer")]
pub struct Verify {
    #[clap(flatten)]
    pub contract_address_or_class_hash: ContractAddressOrClassHashGroup,

    /// Class name of the contract to be verified. Either this or class hash should be provided.
    #[clap(short, long)]
    pub class_name: String,

    /// Where you want your contract to be verified
    #[clap(short, long, value_enum, default_value_t = Verifier::Walnut)]
    pub verifier: Verifier,

    /// The network in which the contract is deployed
    #[clap(short, long, value_enum)]
    pub network: Network,

    /// Automatic yes to confirmation prompts for verification
    #[clap(long, default_value = "false")]
    pub confirm_verification: bool,

    /// Optionally specify package with the contract to be verified
    #[clap(long)]
    pub package: Option<String>,

    // Custom api to be used as a verifier's base url.
    #[clap(long)]
    pub custom_base_api_url: Option<String>,
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
    contract_address: Option<Felt>,
    class_hash: Option<Felt>,
    class_name: String,
    verifier: Verifier,
    network: Network,
    confirm_verification: bool,
    custom_base_api_url: Option<String>,
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

    // Build JSON Payload for the verification request
    // get the parent dir of the manifest path
    let workspace_dir = manifest_path
        .parent()
        .ok_or(anyhow!("Failed to obtain workspace dir"))?;

    match verifier {
        Verifier::Walnut => {
            let walnut = WalnutVerificationInterface::new(network, custom_base_api_url);
            walnut
                .verify(
                    workspace_dir.to_path_buf(),
                    contract_address,
                    class_hash,
                    class_name,
                )
                .await
        }
        Verifier::Voyager => {
            let voyager = VoyagerVerificationInterface::new(network, custom_base_api_url);
            voyager
                .verify(
                    workspace_dir.to_path_buf(),
                    contract_address,
                    class_hash,
                    class_name,
                )
                .await
        }
    }
}
