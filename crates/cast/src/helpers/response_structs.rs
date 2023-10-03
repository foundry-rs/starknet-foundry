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
    pub max_fee: FieldElement,
    pub add_profile: String,
}

#[derive(Serialize)]
pub struct AccountAddResponse {
    pub add_profile: String,
}

#[derive(Serialize)]
pub struct AccountDeleteResponse {
    pub result: String,
}

#[derive(Serialize)]
pub struct MulticallNewResponse {
    pub path: Utf8PathBuf,
    pub content: String,
}
