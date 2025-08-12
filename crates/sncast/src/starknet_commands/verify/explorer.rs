use anyhow::Result;
use camino::Utf8PathBuf;
use foundry_ui::UI;
use serde::Serialize;
use sncast::{Network, response::verify::VerifyResponse};
use starknet::providers::{JsonRpcClient, jsonrpc::HttpTransport};

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum ContractIdentifier {
    ClassHash { class_hash: String },
    Address { contract_address: String },
}

#[derive(Serialize, Debug)]
pub struct VerificationPayload {
    pub contract_name: String,
    #[serde(flatten)]
    pub identifier: ContractIdentifier,
    pub source_code: serde_json::Value,
}

#[async_trait::async_trait]
pub trait VerificationInterface<'a>: Sized {
    fn new(
        network: Network,
        workspace_dir: Utf8PathBuf,
        provider: &'a JsonRpcClient<HttpTransport>,
        ui: &'a UI,
    ) -> Result<Self>;
    async fn verify(
        &self,
        identifier: ContractIdentifier,
        contract_name: String,
        package: Option<String>,
        test_files: bool,
        ui: &UI,
    ) -> Result<VerifyResponse>;
    fn gen_explorer_url(&self) -> String;
}
