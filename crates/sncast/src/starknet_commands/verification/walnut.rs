use super::base::{BaseVerificationInterface, VerificationInterface, VerificationPayload};
use anyhow::Result;
use async_trait::async_trait;
use camino::Utf8PathBuf;
use sncast::response::structs::VerifyResponse;
use sncast::Network;
use starknet::core::types::Felt;
use std::env;

pub struct WalnutVerificationInterface {
    base: BaseVerificationInterface,
}

#[async_trait]
impl VerificationInterface for WalnutVerificationInterface {
    fn new(network: Network, workspace_dir: Utf8PathBuf) -> Self {
        WalnutVerificationInterface {
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
        let url = self.gen_explorer_url();
        self.base.send_verification_request(url, payload).await
    }

    fn gen_explorer_url(&self) -> String {
        let api_base_url =
            env::var("WALNUT_API_URL").unwrap_or_else(|_| "https://api.walnut.dev".to_string());
        let path = match self.base.network {
            Network::Mainnet => "/v1/sn_main/verify",
            Network::Sepolia => "/v1/sn_sepolia/verify",
        };
        format!("{api_base_url}{path}")
    }
}
