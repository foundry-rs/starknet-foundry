use super::base::{BaseVerificationInterface, VerificationInterface, VerificationPayload};
use anyhow::Result;
use async_trait::async_trait;
use camino::Utf8PathBuf;
use sncast::response::structs::VerifyResponse;
use sncast::Network;
use starknet::core::types::Felt;
use std::env;

pub struct VoyagerVerificationInterface {
    base: BaseVerificationInterface,
}

#[async_trait]
impl VerificationInterface for VoyagerVerificationInterface {
    fn new(network: Network, workspace_dir: Utf8PathBuf) -> Self {
        VoyagerVerificationInterface {
            base: BaseVerificationInterface {
                network,
                workspace_dir,
            },
        }
    }

    async fn verify(
        &self,
        contract_address: Option<Felt>,
        class_hash: Option<Felt>,
        class_name: String,
    ) -> Result<VerifyResponse> {
        let file_data = self.base.read_workspace_files()?;
        let source_code = serde_json::Value::Object(file_data);
        let payload = VerificationPayload {
            class_name,
            contract_address,
            class_hash,
            source_code,
        };
        let url = self.gen_explorer_url()?;
        self.base.send_verification_request(url, payload).await
    }

    fn gen_explorer_url(&self) -> Result<String> {
        let custom_api_url = env::var("VOYAGER_API_URL");
        if let Ok(custom_api_url) = custom_api_url {
            return Ok(custom_api_url);
        }

        let api_verification_url = match self.base.network {
            Network::Mainnet => "https://api.voyager.online/beta/class-verify-v2",
            Network::Sepolia => "https://sepolia-api.voyager.online/beta/class-verify-v2",
        };

        Ok(api_verification_url.to_string())
    }
}
