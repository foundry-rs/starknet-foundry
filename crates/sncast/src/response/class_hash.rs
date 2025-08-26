use cairo_vm::Felt252;
use conversions::{serde::serialize::CairoSerialize, string::IntoHexStr};
use foundry_ui::{Message, styling};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::response::{cast_message::SncastMessage, command::CommandResponse};

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct ClassHashResponse {
    pub class_hash: PaddedFelt,
}

impl CommandResponse for ClassHashResponse {}

impl Message for SncastMessage<ClassHashResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
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
