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
pub struct MulticallNewResponse {
    pub path: Utf8PathBuf,
    pub content: String,
}

pub enum VerificationStatus {
    OK,
    Error,
}

impl Serialize for VerificationStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            VerificationStatus::OK => serializer.serialize_bool(true),
            VerificationStatus::Error => serializer.serialize_bool(false),
        }
    }
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub verification_status: VerificationStatus,
    pub errors: Option<String>,
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
