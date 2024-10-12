use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use camino::Utf8PathBuf;
use reqwest::StatusCode;
use serde::Serialize;
use sncast::{helpers::configuration::CastConfig, response::structs::VerifyResponse};
use starknet::core::types::Felt;
use std::ffi::OsStr;
use walkdir::WalkDir;

#[async_trait]
pub trait VerificationInterface {
    fn get_workspace_dir(&self) -> Utf8PathBuf;

    fn gen_explorer_url(&self, config: CastConfig) -> String;

    fn read_workspace_files(&self) -> Result<serde_json::Map<String, serde_json::Value>> {
        // Read all files name along with their contents in a JSON format
        // in the workspace dir recursively
        // key is the file name and value is the file content
        let mut file_data = serde_json::Map::new();

        // Recursively read files and their contents in workspace directory
        let workspace_dir = self.get_workspace_dir();
        for entry in WalkDir::new(workspace_dir.clone()).follow_links(true) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == OsStr::new("cairo") || extension == OsStr::new("toml") {
                        let relative_path = path.strip_prefix(workspace_dir.clone())?;
                        let file_content = std::fs::read_to_string(path)?;
                        file_data.insert(
                            relative_path.to_string_lossy().into_owned(),
                            serde_json::Value::String(file_content),
                        );
                    }
                }
            }
        }
        Ok(file_data)
    }

    async fn send_verification_request(
        &self,
        url: String,
        payload: VerificationPayload,
    ) -> Result<VerifyResponse> {
        let json_payload = serde_json::to_string(&payload)?;
        let client = reqwest::Client::new();
        let api_res = client
            .post(url)
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

    async fn verify(
        &self,
        config: &CastConfig,
        contract_address: Option<Felt>,
        class_hash: Option<Felt>,
        class_name: String,
    ) -> Result<VerifyResponse> {
        let file_data = self.read_workspace_files()?;
        let source_code = serde_json::Value::Object(file_data);
        let payload = VerificationPayload {
            class_name,
            contract_address,
            class_hash,
            source_code,
        };
        let url = self.gen_explorer_url(config.clone());
        self.send_verification_request(url, payload).await
    }
}

#[derive(Serialize, Debug)]
pub struct VerificationPayload {
    pub class_name: String,
    pub contract_address: Option<Felt>,
    pub class_hash: Option<Felt>,
    pub source_code: serde_json::Value,
}
