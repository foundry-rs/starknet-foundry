use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use foundry_ui::UI;
use reqwest::StatusCode;
use sncast::Network;
use sncast::response::verify::VerifyResponse;
use starknet::providers::{JsonRpcClient, jsonrpc::HttpTransport};
use std::env;
use std::ffi::OsStr;
use walkdir::WalkDir;

use super::explorer::{ContractIdentifier, VerificationInterface, VerificationPayload};

pub struct WalnutVerificationInterface {
    network: Network,
    workspace_dir: Utf8PathBuf,
}

#[async_trait::async_trait]
impl VerificationInterface<'_> for WalnutVerificationInterface {
    fn new(
        network: Network,
        workspace_dir: Utf8PathBuf,
        _provider: &JsonRpcClient<HttpTransport>,
        _ui: &UI,
    ) -> Result<Self> {
        Ok(WalnutVerificationInterface {
            network,
            workspace_dir,
        })
    }

    async fn verify(
        &self,
        identifier: ContractIdentifier,
        contract_name: String,
        _package: Option<String>,
        _ui: &UI,
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
            contract_name,
            identifier,
            source_code,
        };

        // Serialize the payload to a JSON string for the POST request
        let json_payload = serde_json::to_string(&payload)?;

        // Send the POST request to the explorer
        let client = reqwest::Client::new();
        let api_res = client
            .post(self.gen_explorer_url())
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
            Err(anyhow::anyhow!("{}", message))
        }
    }

    fn gen_explorer_url(&self) -> String {
        let api_base_url =
            env::var("VERIFIER_API_URL").unwrap_or_else(|_| "https://api.walnut.dev".to_string());
        let path = match self.network {
            Network::Mainnet => "/v1/sn_main/verify",
            Network::Sepolia => "/v1/sn_sepolia/verify",
        };
        format!("{api_base_url}{path}")
    }
}
