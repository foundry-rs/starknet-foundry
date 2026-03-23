use crate::helpers::block_explorer::LinkProvider;
use crate::response::cast_message::SncastCommandMessage;
use crate::response::declare::DeclareTransactionResponse;
use crate::response::dry_run::DryRunResponse;
use crate::response::explorer_link::OutputLink;
use conversions::string::IntoPaddedHexStr;
use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::styling;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum DeployResponse {
    Standard(StandardDeployResponse),
    WithDeclare(DeployResponseWithDeclare),
}

impl SncastCommandMessage for DeployResponse {
    fn text(&self) -> String {
        match &self {
            DeployResponse::Standard(response) => response.text(),
            DeployResponse::WithDeclare(response) => response.text(),
        }
    }

    fn json(&self) -> serde_json::Value {
        match &self {
            DeployResponse::Standard(response) => match response {
                StandardDeployResponse::Transaction(transaction_response) => {
                    serde_json::to_value(transaction_response)
                        .expect("Should be serializable to JSON")
                }
                StandardDeployResponse::DryRun(dry_run_response) => dry_run_response.json(),
            },
            DeployResponse::WithDeclare(response) => {
                serde_json::to_value(response).expect("Should be serializable to JSON")
            }
        }
    }
}

impl OutputLink for DeployResponse {
    const TITLE: &'static str = "deployment";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        match self {
            DeployResponse::Standard(deploy) => match deploy {
                StandardDeployResponse::Transaction(response) => formatdoc!(
                    "
                        contract: {}
                        transaction: {}
                        ",
                    provider.contract(response.contract_address),
                    provider.transaction(response.transaction_hash)
                ),
                StandardDeployResponse::DryRun(_) => {
                    "No links available for fee estimation".to_string()
                }
            },
            DeployResponse::WithDeclare(deploy_with_declare) => {
                formatdoc!(
                    "
                    contract: {}
                    class: {}
                    deploy transaction: {}
                    declare transaction: {}
                    ",
                    provider.contract(deploy_with_declare.contract_address),
                    provider.class(deploy_with_declare.class_hash),
                    provider.transaction(deploy_with_declare.deploy_transaction_hash),
                    provider.transaction(deploy_with_declare.declare_transaction_hash),
                )
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub enum StandardDeployResponse {
    Transaction(StandardDeployTransactionResponse),
    DryRun(DryRunResponse),
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct StandardDeployTransactionResponse {
    pub contract_address: PaddedFelt,
    pub transaction_hash: PaddedFelt,
}

impl StandardDeployResponse {
    fn text(&self) -> String {
        match self {
            StandardDeployResponse::Transaction(response) => styling::OutputBuilder::new()
                .success_message("Deployment completed")
                .blank_line()
                .field(
                    "Contract Address",
                    &response.contract_address.into_padded_hex_str(),
                )
                .field(
                    "Transaction Hash",
                    &response.transaction_hash.into_padded_hex_str(),
                )
                .build(),
            StandardDeployResponse::DryRun(response) => response.text(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct DeployResponseWithDeclare {
    contract_address: PaddedFelt,
    class_hash: PaddedFelt,
    deploy_transaction_hash: PaddedFelt,
    declare_transaction_hash: PaddedFelt,
}

impl DeployResponseWithDeclare {
    #[must_use]
    pub fn from_responses(
        deploy: &StandardDeployResponse,
        declare: &DeclareTransactionResponse,
    ) -> Self {
        match deploy {
            StandardDeployResponse::Transaction(deploy_response) => Self {
                contract_address: deploy_response.contract_address,
                class_hash: declare.class_hash,
                deploy_transaction_hash: deploy_response.transaction_hash,
                declare_transaction_hash: declare.transaction_hash,
            },
            StandardDeployResponse::DryRun(_) => {
                unreachable!("Cannot create DeployResponseWithDeclare from a dry run response")
            }
        }
    }
}

impl DeployResponseWithDeclare {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Deployment completed")
            .blank_line()
            .field(
                "Contract Address",
                &self.contract_address.into_padded_hex_str(),
            )
            .field("Class Hash", &self.class_hash.into_padded_hex_str())
            .field(
                "Declare Transaction Hash",
                &self.declare_transaction_hash.into_padded_hex_str(),
            )
            .field(
                "Deploy Transaction Hash",
                &self.deploy_transaction_hash.into_padded_hex_str(),
            )
            .build()
    }
}
