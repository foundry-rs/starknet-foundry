use super::command::CommandResponse;
use crate::helpers::block_explorer::LinkProvider;
use crate::response::cast_message::{SncastMessage, SncastTextMessage};
use crate::response::declare::DeclareTransactionResponse;
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

impl CommandResponse for DeployResponse {}

impl SncastTextMessage for SncastMessage<DeployResponse> {
    fn text(&self) -> String {
        match &self.command_response {
            DeployResponse::Standard(response) => response.text(),
            DeployResponse::WithDeclare(response) => response.text(),
        }
    }
}

impl OutputLink for DeployResponse {
    const TITLE: &'static str = "deployment";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        match self {
            DeployResponse::Standard(deploy) => {
                formatdoc!(
                    "
                    contract: {}
                    transaction: {}
                    ",
                    provider.contract(deploy.contract_address),
                    provider.transaction(deploy.transaction_hash)
                )
            }
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
pub struct StandardDeployResponse {
    pub contract_address: PaddedFelt,
    pub transaction_hash: PaddedFelt,
}

impl StandardDeployResponse {
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
