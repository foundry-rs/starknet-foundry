use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize, string::IntoHexStr};
use foundry_ui::{Message, styling};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::response::{cast_message::SncastMessage, command::CommandResponse};

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct ClassHashGeneratedResponse {
    pub class_hash: PaddedFelt,
}

impl CommandResponse for ClassHashGeneratedResponse {}

impl Message for SncastMessage<ClassHashGeneratedResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Class Hash generated")
            .blank_line()
            .field(
                "Class Hash",
                &self.command_response.class_hash.into_hex_string(),
            )
            .build()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::to_value(&self.command_response).unwrap_or_else(|err| {
            json!({
                "error": "Failed to serialize response",
                "command": self.command,
                "details": err.to_string()
            })
        })
    }
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct ContractNotFound {
    contract: PaddedFelt,
}

impl CommandResponse for ContractNotFound {}

impl Message for SncastMessage<ContractNotFound> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Contract class not found")
            .blank_line()
            .field(
                "Class Name",
                &self.command_response.contract.into_hex_string(),
            )
            .build()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::to_value(&self.command_response).unwrap_or_else(|err| {
            json!({
                "error": "Failed to serialize response",
                "command": self.command,
                "details": err.to_string()
            })
        })
    }
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
#[serde(tag = "status")]
pub enum ClassHashResponse {
    ContractNotFound(ContractNotFound),
    #[serde(untagged)]
    Success(ClassHashGeneratedResponse),
}

impl CommandResponse for ClassHashResponse {}

impl Message for SncastMessage<ClassHashResponse> {
    fn text(&self) -> String {
        match &self.command_response {
            ClassHashResponse::ContractNotFound(response) => styling::OutputBuilder::new()
                .success_message("Contract class not found")
                .blank_line()
                .field("Class Name", &response.contract.into_hex_string())
                .build(),
            ClassHashResponse::Success(response) => styling::OutputBuilder::new()
                .success_message("Class Hash generated")
                .blank_line()
                .field("Class Hash", &response.class_hash.into_hex_string())
                .build(),
        }
    }

    fn json(&self) -> serde_json::Value {
        serde_json::to_value(&self.command_response).unwrap_or_else(|err| {
            json!({
                "error": "Failed to serialize response",
                "command": self.command,
                "details": err.to_string()
            })
        })
    }
}
