use camino::Utf8PathBuf;
use conversions::serde::serialize::CairoSerialize;
use indoc::formatdoc;
use serde::{Deserialize, Serialize, Serializer};
use starknet::core::types::FieldElement;

use super::explorer_link::OutputLink;

pub struct Decimal(pub u64);

#[derive(Clone, Debug, Deserialize, CairoSerialize, PartialEq)]
pub struct Felt(pub FieldElement);

impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl Serialize for Felt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let val = self.0;
        serializer.serialize_str(&format!("{val:#x}"))
    }
}

fn serialize_as_decimal<S>(value: &Felt, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let val = value.0;
    serializer.serialize_str(&format!("{val:#}"))
}

pub trait CommandResponse: Serialize {}

#[derive(Serialize, CairoSerialize, Clone)]
pub struct CallResponse {
    pub response: Vec<Felt>,
}
impl CommandResponse for CallResponse {}

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct InvokeResponse {
    pub transaction_hash: Felt,
}
impl CommandResponse for InvokeResponse {}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct DeployResponse {
    pub contract_address: Felt,
    pub transaction_hash: Felt,
}
impl CommandResponse for DeployResponse {}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct DeclareResponse {
    pub class_hash: Felt,
    pub transaction_hash: Felt,
}
impl CommandResponse for DeclareResponse {}

#[derive(Serialize)]
pub struct AccountCreateResponse {
    pub address: Felt,
    #[serde(serialize_with = "crate::response::structs::serialize_as_decimal")]
    pub max_fee: Felt,
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
    pub accounts_file_path: Option<Utf8PathBuf>,
    pub keystore: Option<Utf8PathBuf>,
    pub wait_timeout: Option<Decimal>,
    pub wait_retry_interval: Option<Decimal>,
}
impl CommandResponse for ShowConfigResponse {}

#[derive(Serialize, Debug)]
pub struct ScriptRunResponse {
    pub status: String,
    pub message: Option<String>,
}

impl CommandResponse for ScriptRunResponse {}

#[derive(Serialize)]
pub struct ScriptInitResponse {
    pub message: String,
}

impl CommandResponse for ScriptInitResponse {}

#[derive(Serialize, CairoSerialize)]
pub enum FinalityStatus {
    Received,
    Rejected,
    AcceptedOnL2,
    AcceptedOnL1,
}

#[derive(Serialize, CairoSerialize)]
pub enum ExecutionStatus {
    Succeeded,
    Reverted,
}

#[derive(Serialize, CairoSerialize)]
pub struct TransactionStatusResponse {
    pub finality_status: FinalityStatus,
    pub execution_status: Option<ExecutionStatus>,
}

impl CommandResponse for TransactionStatusResponse {}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub message: String,
}

impl CommandResponse for VerifyResponse {}

impl OutputLink for InvokeResponse {
    const TITLE: &'static str = "invocation";

    fn format_links(&self, service: &str) -> String {
        format!("transaction: {service}/{:x}", self.transaction_hash.0)
    }
}

impl OutputLink for DeployResponse {
    const TITLE: &'static str = "deployment";

    fn format_links(&self, service: &str) -> String {
        formatdoc!(
            "
            contract: {service}/{:x}
            transaction: {service}/{:x}
            ",
            self.contract_address.0,
            self.transaction_hash.0
        )
    }
}

impl OutputLink for DeclareResponse {
    const TITLE: &'static str = "declaration";

    fn format_links(&self, service: &str) -> String {
        formatdoc!(
            "
            class: {service}/{:x}
            transaction: {service}/{:x}
            ",
            self.class_hash.0,
            self.transaction_hash.0
        )
    }
}

impl OutputLink for AccountCreateResponse {
    const TITLE: &'static str = "account creation";

    fn format_links(&self, service: &str) -> String {
        format!("account: {service}/{:x}", self.address.0)
    }
}
