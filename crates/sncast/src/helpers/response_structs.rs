use camino::Utf8PathBuf;
use serde::Serialize;
use starknet::core::types::FieldElement;


pub trait CommandResponse: Serialize {}


#[derive(Serialize, Clone)]
pub struct CallResponse {
    pub response: Vec<FieldElement>,
}
impl CommandResponse for CallResponse {}

#[derive(Serialize, Clone)]
pub struct InvokeResponse {
    pub transaction_hash: FieldElement,
}
impl CommandResponse for InvokeResponse {}

#[derive(Serialize)]
pub struct DeployResponse {
    pub contract_address: FieldElement,
    pub transaction_hash: FieldElement,
}
impl CommandResponse for DeployResponse {}


#[derive(Serialize)]
pub struct DeclareResponse {
    pub class_hash: FieldElement,
    pub transaction_hash: FieldElement,
}
impl CommandResponse for DeclareResponse {}



#[derive(Serialize)]
pub struct AccountCreateResponse {
    pub address: FieldElement,
    pub max_fee: u64,
    pub add_profile: String,
    pub message: String,
}

impl CommandResponse for AccountCreateResponse {}

#[derive(Serialize)]
pub struct AccountAddResponse {
    pub add_profile: String,
}

impl CommandResponse for AccountAddResponse {}

#[derive(Serialize)]
pub struct AccountDeleteResponse {
    pub result: String,
    pub scarb_result: String,
}

impl CommandResponse for AccountDeleteResponse {}

#[derive(Serialize)]
pub struct MulticallNewResponse {
    pub path: Utf8PathBuf,
    pub content: String,
}
impl CommandResponse for MulticallNewResponse {}

#[derive(Serialize)]
pub struct ShowConfigResponse {
    pub profile: Option<String>,
    pub chain_id: String,
    pub rpc_url: Option<String>,
    pub account: Option<String>,
    pub scarb_path: Option<Utf8PathBuf>,
    pub accounts_file_path: Option<Utf8PathBuf>,
    pub keystore: Option<Utf8PathBuf>,
    pub wait_timeout: Option<u16>,
    pub wait_retry_interval: Option<u8>,
}
impl CommandResponse for ShowConfigResponse {}

#[derive(Serialize)]
pub struct ScriptResponse {
    pub status: String,
    pub msg: Option<String>,
}

impl CommandResponse for ScriptResponse {}
