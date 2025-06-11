use conversions::string::IntoHexStr;
use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::Message;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{command::CommandResponse, explorer_link::OutputLink};
use crate::helpers::block_explorer::LinkProvider;
use crate::response::cast_message::SncastMessage;
use foundry_ui::styling;

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct DeclareTransactionResponse {
    pub class_hash: PaddedFelt,
    pub transaction_hash: PaddedFelt,
}

impl CommandResponse for DeclareTransactionResponse {}

impl Message for SncastMessage<DeclareTransactionResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Declaration completed successfully")
            .blank_line()
            .field(
                "Class Hash",
                &self.command_response.class_hash.into_hex_string(),
            )
            .field(
                "Transaction Hash",
                &self.command_response.transaction_hash.into_hex_string(),
            )
            .build()
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap()
    }
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct AlreadyDeclaredResponse {
    pub class_hash: PaddedFelt,
}

impl CommandResponse for AlreadyDeclaredResponse {}

impl Message for SncastMessage<AlreadyDeclaredResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Contract class already declared")
            .blank_line()
            .field(
                "Class Hash",
                &self.command_response.class_hash.into_hex_string(),
            )
            .build()
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap()
    }
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
#[serde(tag = "status")]
pub enum DeclareResponse {
    AlreadyDeclared(AlreadyDeclaredResponse),
    #[serde(untagged)]
    Success(DeclareTransactionResponse),
}

impl CommandResponse for DeclareResponse {}

impl Message for SncastMessage<DeclareResponse> {
    fn text(&self) -> String {
        match &self.command_response {
            DeclareResponse::AlreadyDeclared(response) => styling::OutputBuilder::new()
                .success_message("Contract class already declared")
                .blank_line()
                .field("Class Hash", &response.class_hash.into_hex_string())
                .build(),
            DeclareResponse::Success(response) => styling::OutputBuilder::new()
                .success_message("Declaration completed successfully")
                .blank_line()
                .field("Class Hash", &response.class_hash.into_hex_string())
                .field(
                    "Transaction Hash",
                    &response.transaction_hash.into_hex_string(),
                )
                .build(),
        }
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap()
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
