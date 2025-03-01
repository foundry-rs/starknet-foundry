use anyhow::{anyhow, bail, Result};
use camino::Utf8PathBuf;
use clap::{Args, ValueEnum};
use promptly::prompt;
use scarb_api::StarknetContractArtifacts;
use sncast::{response::structs::VerifyResponse, Network};
use starknet_types_core::felt::Felt;
use std::{collections::HashMap, fmt};

pub mod explorer;
pub mod walnut;

use explorer::VerificationInterface;
use walnut::WalnutVerificationInterface;

#[derive(Args)]
#[command(about = "Verify a contract through a block explorer")]
pub struct Verify {
    /// Class hash of a contract to be verified
    #[clap(short = 'g', long)]
    pub class_hash: Option<Felt>,

    /// Address of a contract to be verified
    #[clap(short = 'd', long)]
    pub contract_address: Option<Felt>,

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

impl Verify {
    pub fn validate(&self) -> Result<()> {
        if self.class_hash.is_none() && self.contract_address.is_none() {
            return Err(anyhow!(
                "You must provide either --class-hash or --contract-address."
            ));
        }
        Ok(())
    }
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

    match verifier {
        Verifier::Walnut => {
            let walnut =
                WalnutVerificationInterface::new(verify.network, workspace_dir.to_path_buf());
            walnut
                .verify(verify.class_hash, verify.contract_address, contract_name)
                .await
        }
    }
}
