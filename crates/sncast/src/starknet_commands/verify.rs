use anyhow::{anyhow, Context, Result};
use anyhow::{bail, Ok};
use camino::Utf8PathBuf;
use clap::{Args, ValueEnum};
use promptly::prompt;
use reqwest::StatusCode;
use scarb_api::StarknetContractArtifacts;
use serde::Serialize;
use sncast::response::structs::VerifyResponse;
use sncast::Network;
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::{env, fmt};
use walkdir::WalkDir;

struct WalnutVerificationInterface {
    network: Network,
    workspace_dir: Utf8PathBuf,
}

#[async_trait::async_trait]
trait VerificationInterface {
    fn new(network: Network, workspace_dir: Utf8PathBuf) -> Self;
    async fn verify(
        &self,
        class_hash: Option<Felt>,
        contract_address: Option<Felt>,
        contract_name: String,
    ) -> Result<VerifyResponse>;
    fn gen_explorer_url(&self) -> Result<String>;
}

#[async_trait::async_trait]
impl VerificationInterface for WalnutVerificationInterface {
    fn new(network: Network, workspace_dir: Utf8PathBuf) -> Self {
        WalnutVerificationInterface {
            network,
            workspace_dir,
        }
    }

    async fn verify(
        &self,
        class_hash: Option<Felt>,
        contract_address: Option<Felt>,
        contract_name: String,
    ) -> Result<VerifyResponse> {
        // Read all files name along with their contents in a JSON format
        // in the workspace dir recursively
        // key is the file name and value is the file content
        let mut file_data = serde_json::Map::new();

        // Recursively read files and their contents in workspace directory
        for entry in WalkDir::new(self.workspace_dir.clone()).follow_links(true) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == OsStr::new("cairo") || extension == OsStr::new("toml") {
                        let relative_path = path.strip_prefix(self.workspace_dir.clone())?;
                        let file_content = std::fs::read_to_string(path)?;
                        file_data.insert(
                            relative_path.to_string_lossy().into_owned(),
                            serde_json::Value::String(file_content),
                        );
                    }
                }
            }
        }

        // Serialize the JSON object to a JSON string
        let source_code = serde_json::Value::Object(file_data);

        // Convert contract addresss and class hash from the Felt to String
        let contract_address_str = contract_address.map(|addr| addr.to_fixed_hex_string());
        let class_hash_str = class_hash.map(|hash| hash.to_fixed_hex_string());

        // Create the JSON payload with "contract name," "address," and "source_code" fields
        let payload = VerificationPayload {
            contract_name,
            class_hash: class_hash_str,
            contract_address: contract_address_str,
            source_code,
        };

        // Serialize the payload to a JSON string for the POST request
        let json_payload = serde_json::to_string(&payload)?;

        // Send the POST request to the explorer
        let client = reqwest::Client::new();
        let api_res = client
            .post(self.gen_explorer_url()?)
            .header("Content-Type", "application/json")
            .body(json_payload)
            .send()
            .await
            .context("Failed to send request to verifier API")?;

        if api_res.status() == StatusCode::OK {
            let message = api_res
                .text()
                .await
                .context("Failed to read verifier API response")?;
            Ok(VerifyResponse { message })
        } else {
            let message = api_res.text().await.context("Failed to verify contract")?;
            Err(anyhow!(message))
        }
    }

    fn gen_explorer_url(&self) -> Result<String> {
        let api_base_url =
            env::var("WALNUT_API_URL").unwrap_or_else(|_| "https://api.walnut.dev".to_string());
        let path = match self.network {
            Network::Mainnet => "/v1/sn_main/verify",
            Network::Sepolia => "/v1/sn_sepolia/verify",
        };
        Ok(format!("{api_base_url}{path}"))
    }
}

#[derive(Args)]
#[command(about = "Verify a contract through a block explorer")]
pub struct Verify {
    /// Class hash of a contract to be verified
    #[clap(short = 'x', long)]
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

#[derive(Serialize, Debug)]
struct VerificationPayload {
    contract_name: String,
    class_hash: Option<String>,
    contract_address: Option<String>,
    source_code: serde_json::Value,
}

pub async fn verify(
    verify: Verify,
    manifest_path: &Utf8PathBuf,
    artifacts: &HashMap<String, StarknetContractArtifacts>,
) -> Result<VerifyResponse> {
    // Let's ask confirmation
    if !verify.confirm_verification {
        let verifier = verify.verifier;
        let prompt_text = format!("\n\tYou are about to submit the entire workspace code to the third-party verifier at {verifier}.\n\n\tImportant: Make sure your project does not include sensitive information like private keys. The snfoundry.toml file will be uploaded. Keep the keystore outside the project to prevent it from being uploaded.\n\n\tAre you sure you want to proceed? (Y/n)");
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

    match verify.verifier {
        Verifier::Walnut => {
            let walnut =
                WalnutVerificationInterface::new(verify.network, workspace_dir.to_path_buf());
            walnut
                .verify(verify.class_hash, verify.contract_address, contract_name)
                .await
        }
    }
}
