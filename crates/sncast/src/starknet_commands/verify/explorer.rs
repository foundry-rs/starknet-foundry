use anyhow::Result;
use camino::Utf8PathBuf;
use serde::Serialize;
use sncast::{Network, response::structs::VerifyResponse};
use starknet_types_core::felt::Felt;

#[derive(Serialize, Debug)]
pub struct VerificationPayload {
    pub contract_name: String,
    pub contract_address: String,
    pub source_code: serde_json::Value,
}

#[async_trait::async_trait]
pub trait VerificationInterface {
    fn new(network: Network, workspace_dir: Utf8PathBuf) -> Self;

    async fn verify(&self, contract_address: Felt, contract_name: String)
    -> Result<VerifyResponse>;

    fn gen_explorer_url(&self) -> Result<String>;
}
