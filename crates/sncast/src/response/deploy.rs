use super::command::CommandResponse;
use crate::helpers::block_explorer::LinkProvider;
use crate::response::cast_message::SncastMessage;
use crate::response::declare::DeclareTransactionResponse;
use crate::response::explorer_link::OutputLink;
use conversions::string::IntoPaddedHexStr;
use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::Message;
use foundry_ui::styling;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum DeployResponseKind {
    Deploy(DeployResponse),
    DeployWithDeclare(DeployResponseWithDeclare),
}

impl CommandResponse for DeployResponseKind {}

impl Message for SncastMessage<DeployResponseKind> {
    fn text(&self) -> String {
        match &self.command_response {
            DeployResponseKind::Deploy(response) => response.text(),
            DeployResponseKind::DeployWithDeclare(response) => response.text(),
        }
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap_or_else(|err| {
            json!({
                "error": "Failed to serialize response",
                "command": self.command,
                "details": err.to_string()
            })
        })
    }
}

impl OutputLink for DeployResponseKind {
    const TITLE: &'static str = "deployment";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        match self {
            DeployResponseKind::Deploy(deploy) => {
                formatdoc!(
                    "
                    contract: {}
                    transaction: {}
                    ",
                    provider.contract(deploy.contract_address),
                    provider.transaction(deploy.transaction_hash)
                )
            }
            DeployResponseKind::DeployWithDeclare(deploy_with_declare) => {
                formatdoc!(
                    "
                    contract: {}
                    class: {}
                    declare transaction: {}
                    deploy transaction: {}
                    ",
                    provider.contract(deploy_with_declare.contract_address),
                    provider.class(deploy_with_declare.class_hash),
                    provider.transaction(deploy_with_declare.declare_transaction_hash),
                    provider.transaction(deploy_with_declare.deploy_transaction_hash)
                )
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct DeployResponse {
    pub contract_address: PaddedFelt,
    pub transaction_hash: PaddedFelt,
}

impl DeployResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Deployment completed")
            .blank_line()
            .field(
                "Contract Address",
                &self.contract_address.into_padded_hex_str(),
            )
            .field(
                "Transaction Hash",
                &self.transaction_hash.into_padded_hex_str(),
            )
            .build()
    }
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct DeployResponseWithDeclare {
    pub contract_address: PaddedFelt,
    pub class_hash: PaddedFelt,
    pub deploy_transaction_hash: PaddedFelt,
    pub declare_transaction_hash: PaddedFelt,
}

impl DeployResponseWithDeclare {
    #[must_use]
    pub fn from_responses(deploy: &DeployResponse, declare: &DeclareTransactionResponse) -> Self {
        Self {
            contract_address: deploy.contract_address,
            class_hash: declare.class_hash,
            deploy_transaction_hash: deploy.transaction_hash,
            declare_transaction_hash: declare.transaction_hash,
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
