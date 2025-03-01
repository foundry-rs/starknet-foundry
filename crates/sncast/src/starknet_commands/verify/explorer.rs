use anyhow::Result;
use camino::Utf8PathBuf;
use serde::Serialize;
use sncast::{response::structs::VerifyResponse, Network};
use starknet_types_core::felt::Felt;

#[derive(Serialize, Debug)]
pub struct VerificationPayload {
    pub contract_name: String,
    pub class_hash: Option<String>,
    pub contract_address: Option<String>,
    pub source_code: serde_json::Value,
}

#[async_trait::async_trait]
pub trait VerificationInterface {
    fn new(network: Network, workspace_dir: Utf8PathBuf) -> Self;
    async fn verify(
        &self,
        class_hash: Option<Felt>,
        contract_address: Option<Felt>,
        contract_name: String,
    ) -> Result<VerifyResponse>;
    fn gen_explorer_url(&self) -> Result<String>;
}
