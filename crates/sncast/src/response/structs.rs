use camino::Utf8PathBuf;
use serde::{Serialize, Serializer};
use starknet::core::types::FieldElement;

pub struct Decimal(pub u64);

#[derive(Clone)]
pub struct Hex(pub FieldElement);

impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl Serialize for Hex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = self.0;
        serializer.serialize_str(&format!("{val:#x}"))
    }
}

pub trait CommandResponse: Serialize {}

#[derive(Serialize, Clone)]
pub struct CallResponse {
    pub response: Vec<Hex>,
}
impl CommandResponse for CallResponse {}

#[derive(Serialize, Clone)]
pub struct InvokeResponse {
    pub transaction_hash: Hex,
}
impl CommandResponse for InvokeResponse {}

#[derive(Serialize)]
pub struct DeployResponse {
    pub contract_address: Hex,
    pub transaction_hash: Hex,
}
impl CommandResponse for DeployResponse {}

#[derive(Serialize)]
pub struct DeclareResponse {
    pub class_hash: Hex,
    pub transaction_hash: Hex,
}
impl CommandResponse for DeclareResponse {}

#[derive(Serialize)]
pub struct AccountCreateResponse {
    pub address: Hex,
    pub max_fee: Decimal,
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
    pub wait_timeout: Option<Decimal>,
    pub wait_retry_interval: Option<Decimal>,
}
impl CommandResponse for ShowConfigResponse {}

#[derive(Serialize, Debug)]
pub struct ScriptResponse {
    pub status: String,
    pub msg: Option<String>,
}

impl CommandResponse for ScriptResponse {}
