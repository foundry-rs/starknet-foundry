use cairo_felt::Felt252;
use camino::Utf8PathBuf;
use conversions::felt252::SerializeAsFelt252Vec;
use conversions::FromConv;
use serde::{Deserialize, Serialize, Serializer};
use starknet::core::types::FieldElement;

pub struct Decimal(pub u64);

#[derive(Clone, Debug, Deserialize, PartialEq)]
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

#[derive(Serialize, Clone)]
pub struct CallResponse {
    pub response: Vec<Felt>,
}
impl CommandResponse for CallResponse {}

impl SerializeAsFelt252Vec for CallResponse {
    fn serialize_into_felt252_vec(self, output: &mut Vec<Felt252>) {
        output.push(Felt252::from(self.response.len()));
        output.extend(self.response.iter().map(|el| Felt252::from_(el.0)));
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InvokeResponse {
    pub transaction_hash: Felt,
}
impl CommandResponse for InvokeResponse {}

impl SerializeAsFelt252Vec for InvokeResponse {
    fn serialize_into_felt252_vec(self, output: &mut Vec<Felt252>) {
        output.push(Felt252::from_(self.transaction_hash.0));
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct DeployResponse {
    pub contract_address: Felt,
    pub transaction_hash: Felt,
}
impl CommandResponse for DeployResponse {}

impl SerializeAsFelt252Vec for DeployResponse {
    fn serialize_into_felt252_vec(self, output: &mut Vec<Felt252>) {
        output.extend([
            Felt252::from_(self.contract_address.0),
            Felt252::from_(self.transaction_hash.0),
        ]);
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct DeclareResponse {
    pub class_hash: Felt,
    pub transaction_hash: Felt,
}
impl CommandResponse for DeclareResponse {}

impl SerializeAsFelt252Vec for DeclareResponse {
    fn serialize_into_felt252_vec(self, output: &mut Vec<Felt252>) {
        output.extend([
            Felt252::from_(self.class_hash.0),
            Felt252::from_(self.transaction_hash.0),
        ]);
    }
}

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
