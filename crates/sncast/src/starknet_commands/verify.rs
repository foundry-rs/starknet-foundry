use anyhow::Ok;
use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use reqwest::StatusCode;
use serde::Serialize;
use sncast::response::structs::{VerificationStatus, VerifyResponse};
use starknet::core::types::FieldElement;
use std::ffi::OsStr;
use walkdir::WalkDir;

struct WalnutVerificationInterface {
    network: String,
    workspace_dir: Utf8PathBuf,
}

#[async_trait::async_trait]
trait VerificationInterface {
    fn new(network: String, workspace_dir: Utf8PathBuf) -> Self;
    async fn verify(
        &self,
        contract_address: FieldElement,
        contract_name: String,
    ) -> Result<VerifyResponse>;
    fn gen_explorer_url(&self) -> Result<String>;
}

#[async_trait::async_trait]
impl VerificationInterface for WalnutVerificationInterface {
    fn new(network: String, workspace_dir: Utf8PathBuf) -> Self {
        WalnutVerificationInterface {
            network,
            workspace_dir,
        }
    }

    async fn verify(
        &self,
        contract_address: FieldElement,
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

        // Create the JSON payload with "contract name," "address," and "source_code" fields
        let payload = VerificationPayload {
            contract_name: contract_name.to_string(),
            contract_address: contract_address.to_string(),
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
            .await?;

        let verification_status = match &api_res.status() {
            &StatusCode::OK => VerificationStatus::OK,
            _ => VerificationStatus::Error,
        };

        Ok(VerifyResponse {
            verification_status,
            message: api_res.text().await.ok(),
        })
    }

    fn gen_explorer_url(&self) -> Result<String> {
        let explorer_url: &str = match self.network.as_str() {
            "mainnet" => Ok("http://api.walnut.dev/v1/sn_main/verify"),
            "goerli" => Ok("https://api.walnut.dev/v1/sn_goerli/verify"),
            _ => Err(anyhow!("Unknown network")),
        }?;
        Ok(explorer_url.to_string())
    }
}

#[derive(Args)]
#[command(about = "Verify a contract through a block explorer")]
pub struct Verify {
    /// Address of a contract to be verified
    #[clap(short = 'a', long)]
    pub contract_address: FieldElement,

    /// Name of the contract that is being verified
    #[clap(short, long)]
    pub contract_name: String,

    /// Block explorer to use for the verification
    #[clap(short = 'v', long = "verifier", value_parser = ["walnut"])]
    pub verifier: String,

    /// The network on which block explorer will do the verification
    #[clap(short = 'n', long = "network", value_parser = ["mainnet", "goerli"])]
    pub network: String,
}

#[derive(Serialize, Debug)]
struct VerificationPayload {
    contract_name: String,
    contract_address: String,
    source_code: serde_json::Value,
}

pub async fn verify(
    contract_address: FieldElement,
    contract_name: String,
    verifier: String,
    network: String,
    manifest_path: &Utf8PathBuf,
) -> Result<VerifyResponse> {
    // Build JSON Payload for the verification request
    // get the parent dir of the manifest path
    let workspace_dir = manifest_path
        .parent()
        .ok_or(anyhow!("Failed to obtain workspace dir"))?;

    match verifier.as_str() {
        "walnut" => {
            let walnut = WalnutVerificationInterface::new(network, workspace_dir.to_path_buf());
            walnut
                .verify(contract_address, contract_name)
                .await
                .context("Failed to verify contract")
        }
        _ => Err(anyhow!("Unknown verifier")),
    }
}
