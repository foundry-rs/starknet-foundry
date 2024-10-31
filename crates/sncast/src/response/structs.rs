use super::explorer_link::OutputLink;
use crate::helpers::block_explorer::LinkProvider;
use camino::Utf8PathBuf;
use conversions::serde::serialize::CairoSerialize;
use indoc::formatdoc;
use itertools::Itertools;
use serde::{Deserialize, Serialize, Serializer};
use starknet::core::types::Felt;

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
pub struct AccountImportResponse {
    pub add_profile: String,
}

impl CommandResponse for AccountImportResponse {}

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

#[derive(Serialize)]
pub struct DeclareDeployResponse {
    class_hash: Option<Felt>,
    declare_transaction_hash: Option<Felt>,
    contract_address: Felt,
    deploy_transaction_hash: Felt,
}

impl DeclareDeployResponse {
    #[must_use]
    pub fn new(declare: &Option<DeclareResponse>, deploy: &DeployResponse) -> Self {
        let class_hash = declare.as_ref().map(|it| it.class_hash);
        let declare_transaction_hash = declare.as_ref().map(|it| it.transaction_hash);

        let DeployResponse {
            contract_address,
            transaction_hash: deploy_transaction_hash,
        } = deploy;

        Self {
            class_hash,
            declare_transaction_hash,
            contract_address: *contract_address,
            deploy_transaction_hash: *deploy_transaction_hash,
        }
    }
}

impl CommandResponse for DeclareDeployResponse {}

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

impl OutputLink for DeclareResponse {
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

impl OutputLink for DeclareDeployResponse {
    const TITLE: &'static str = "declaration and deployment";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        let mut links = vec![];

        if let Some(ref class_hash) = self.class_hash {
            links.push(format!("class: {}", provider.class(*class_hash)));
        }

        links.push(format!(
            "contract: {}",
            provider.contract(self.contract_address)
        ));

        if let Some(ref transaction_hash) = self.declare_transaction_hash {
            links.push(format!(
                "declaration transaction: {}",
                provider.class(*transaction_hash)
            ));
        }

        links.push(format!(
            "deployment transaction: {}",
            provider.transaction(self.deploy_transaction_hash)
        ));

        links.iter().join("\n")
    }
}

impl OutputLink for AccountCreateResponse {
    const TITLE: &'static str = "account creation";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!("account: {}", provider.contract(self.address))
    }
}
