use camino::Utf8PathBuf;
use serde::Serialize;
use starknet::core::types::FieldElement;

#[derive(Serialize, Clone)]
pub struct InvokeResponse {
    pub transaction_hash: FieldElement,
}

#[derive(Serialize)]
pub struct DeployResponse {
    pub contract_address: FieldElement,
    pub transaction_hash: FieldElement,
}

#[derive(Serialize)]
pub struct DeclareResponse {
    pub class_hash: FieldElement,
    pub transaction_hash: FieldElement,
}

#[derive(Serialize)]
pub struct CallResponse {
    pub response: String,
}

#[derive(Serialize)]
pub struct AccountCreateResponse {
    pub address: FieldElement,
    pub max_fee: u64,
    pub add_profile: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct AccountAddResponse {
    pub add_profile: String,
}

#[derive(Serialize)]
pub struct AccountDeleteResponse {
    pub result: String,
    pub scarb_result: String,
}

#[derive(Serialize)]
pub struct MulticallNewResponse {
    pub path: Utf8PathBuf,
    pub content: String,
}

#[derive(Serialize)]
pub struct ShowConfigResponse {
    pub profile: Option<String>,
    pub chain_id: String,
    pub rpc_url: Option<String>,
    pub account: Option<String>,
    pub scarb_path: Option<Utf8PathBuf>,
    pub accounts_file_path: Option<Utf8PathBuf>,
    pub keystore: Option<Utf8PathBuf>,
}
