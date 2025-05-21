use super::explorer_link::OutputLink;
use crate::helpers::block_explorer;
use crate::helpers::block_explorer::LinkProvider;
use camino::Utf8PathBuf;
use conversions::serde::serialize::CairoSerialize;
use conversions::{byte_array::ByteArray, padded_felt::PaddedFelt};
use foundry_ui::Message;
use foundry_ui::formats::NumbersFormat;
use foundry_ui::output_value::{Format, OutputValue};
use indoc::formatdoc;
use serde::{Deserialize, Serialize, Serializer};
use starknet_types_core::felt::Felt;

pub struct Decimal(pub u64);

impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

fn serialize_as_decimal<S>(value: &Felt, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{value:#}"))
}

// pub trait CommandResponse: Serialize {}

#[derive(Serialize, CairoSerialize, Clone)]
pub struct CallResponse {
    #[serde(default = "call_response_command")]
    pub command: ByteArray,
    pub response: Vec<Felt>,
}

fn call_response_command() -> ByteArray {
    ByteArray::from("call")
}

impl Message for CallResponse {
    fn text(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized + Serialize,
    {
        let response = OutputValue::Array(
            self.response
                .iter()
                .map(|f| OutputValue::String(f.to_string()).format_with(numbers_format))
                .collect(),
        );
        format!(
            "command: {}
response: {}",
            self.command, response
        )
    }
}

#[derive(Serialize, Clone)]
pub struct TransformedCallResponse {
    pub command: String,
    pub response: String,
    pub response_raw: Vec<Felt>,
}

impl Message for TransformedCallResponse {
    fn text(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized,
    {
        let response = OutputValue::Array(
            self.response_raw
                .iter()
                .map(|f| OutputValue::String(f.to_hex_string()).format_with(numbers_format))
                .collect(),
        );
        format!(
            "command: {}
response: {}
response_raw: {}",
            self.command, self.response, response
        )
    }
}

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct InvokeResponse {
    #[serde(default = "invoke_response_command")]
    pub command: ByteArray,
    pub transaction_hash: PaddedFelt,
}

fn invoke_response_command() -> ByteArray {
    ByteArray::from("invoke")
}

impl Message for InvokeResponse {
    fn text(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized,
    {
        let _ = numbers_format;
        format!(
            "command: {}
transaction_hash: {}",
            self.command, self.transaction_hash
        )
    }
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct DeployResponse {
    pub command: ByteArray,
    pub contract_address: PaddedFelt,
    pub transaction_hash: PaddedFelt,
}
impl Message for DeployResponse {
    fn text(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized,
    {
        let _ = numbers_format;
        format!(
            "command: {}
contract_address: {:#x}
transaction_hash: {:#x}",
            self.command, self.contract_address, self.transaction_hash
        )
    }
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct DeclareTransactionResponse {
    #[serde(default = "declare_transaction_response_command")]
    pub command: ByteArray,
    pub class_hash: PaddedFelt,
    pub transaction_hash: PaddedFelt,
}

fn declare_transaction_response_command() -> ByteArray {
    ByteArray::from("declare")
}

impl Message for DeclareTransactionResponse {
    fn text(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized + Serialize,
    {
        let _ = numbers_format;
        format!(
            "command: {}
class_hash: {:#x}
transaction_hash: {:#x}",
            self.command, self.class_hash, self.transaction_hash
        )
    }
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct AlreadyDeclaredResponse {
    pub class_hash: PaddedFelt,
}

// impl CommandResponse for AlreadyDeclaredResponse {}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
#[serde(tag = "status")]
pub enum DeclareResponse {
    AlreadyDeclared(AlreadyDeclaredResponse),
    #[serde(untagged)]
    Success(DeclareTransactionResponse),
}

// impl CommandResponse for DeclareResponse {}

#[derive(Serialize, Debug)]
pub struct AccountCreateResponse {
    pub command: String,
    pub address: PaddedFelt,
    #[serde(serialize_with = "crate::response::structs::serialize_as_decimal")]
    pub max_fee: Felt,
    pub add_profile: String,
    pub message: String,
}

// impl CommandResponse for AccountCreateResponse {}

impl Message for AccountCreateResponse {
    fn text(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized,
    {
        let max_fee = OutputValue::String(self.max_fee.to_string());
        format!(
            "command: {}
add_profile: {}
address: {:#x}
max_fee: {}
message: {}",
            self.command,
            self.add_profile,
            self.address,
            max_fee.format_with(numbers_format),
            self.message
        )
    }
}

#[derive(Serialize)]
pub struct AccountImportResponse {
    pub command: String,
    pub add_profile: String,
    pub account_name: Option<String>,
}

// impl CommandResponse for AccountImportResponse {}

impl Message for AccountImportResponse {
    fn text(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized,
    {
        let _ = numbers_format;

        if let Some(account_name) = &self.account_name {
            format!(
                "command: {}
account_name: {}
add_profile: {}",
                self.command, account_name, self.add_profile
            )
        } else {
            format!(
                "command: {}
add_profile: {}",
                self.command, self.add_profile
            )
        }
    }
}

#[derive(Serialize)]
pub struct AccountDeleteResponse {
    pub command: String,
    pub result: String,
}

impl Message for AccountDeleteResponse {
    fn text(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized,
    {
        let _ = numbers_format;
        format!(
            "command: {}
result: {}",
            self.command, self.result
        )
    }
}

#[derive(Serialize)]
pub struct MulticallNewResponse {
    pub command: String,
    pub path: Utf8PathBuf,
    pub content: String,
}

impl Message for MulticallNewResponse {
    fn text(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized,
    {
        let _ = numbers_format;
        format!(
            "command: {}
path: {}
content: {}",
            self.command, self.path, self.content
        )
    }
}

#[derive(Serialize)]
pub struct ShowConfigResponse {
    pub profile: Option<String>,
    pub chain_id: Option<String>,
    pub rpc_url: Option<String>,
    pub account: Option<String>,
    pub accounts_file_path: Option<Utf8PathBuf>,
    pub keystore: Option<Utf8PathBuf>,
    pub wait_timeout: Option<Decimal>,
    pub wait_retry_interval: Option<Decimal>,
    pub show_explorer_links: bool,
    pub block_explorer: Option<block_explorer::Service>,
}

impl Message for ShowConfigResponse {}

#[derive(Serialize, Debug)]
pub struct ScriptRunResponse {
    pub command: String,
    pub status: String,
    pub message: Option<String>,
}

impl Message for ScriptRunResponse {
    fn text(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized,
    {
        let _ = numbers_format;

        if let Some(message) = self.message.as_ref() {
            format!(
                "command: {}
status: {}
message: {}",
                self.command, self.status, message
            )
        } else {
            format!(
                "command: {}
status: {}",
                self.command, self.status,
            )
        }
    }
}

#[derive(Serialize)]
pub struct ScriptInitResponse {
    pub command: String,
    pub message: String,
}

impl Message for ScriptInitResponse {
    fn text(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized,
    {
        let _ = numbers_format;
        format!(
            "command: {}
message: {}",
            self.command, self.message
        )
    }
}

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

impl Message for TransactionStatusResponse {}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub command: String,
    pub message: String,
}

impl Message for VerifyResponse {}

impl OutputLink for InvokeResponse {
    const TITLE: &'static str = "invocation";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!(
            "transaction: {}",
            provider.transaction(self.transaction_hash)
        )
    }
}

impl OutputLink for DeployResponse {
    const TITLE: &'static str = "deployment";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        formatdoc!(
            "
            contract: {}
            transaction: {}
            ",
            provider.contract(self.contract_address),
            provider.transaction(self.transaction_hash)
        )
    }
}

impl OutputLink for DeclareTransactionResponse {
    const TITLE: &'static str = "declaration";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        formatdoc!(
            "
            class: {}
            transaction: {}
            ",
            provider.class(self.class_hash),
            provider.transaction(self.transaction_hash)
        )
    }
}

impl OutputLink for AccountCreateResponse {
    const TITLE: &'static str = "account creation";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!("account: {}", provider.contract(self.address))
    }
}

#[derive(Serialize, Debug)]
pub struct ResponseError {
    command: String,
    error: String,
}

impl ResponseError {
    #[must_use]
    pub fn new(command: String, error: String) -> Self {
        Self { command, error }
    }
}

impl Message for ResponseError {
    fn text(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized,
    {
        let _ = numbers_format;
        format!(
            "command: {}
error: {}",
            self.command, self.error
        )
    }
}
