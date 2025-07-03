use super::command::CommandResponse;
use crate::response::cast_message::SncastMessage;
use conversions::string::IntoHexStr;
use foundry_ui::Message;
use foundry_ui::styling;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use starknet_types_core::felt::Felt;

#[derive(Serialize, Clone)]
pub struct TransformedCallResponse {
    pub response: String,
    pub response_raw: Vec<Felt>,
}

impl CommandResponse for TransformedCallResponse {}

impl Message for SncastMessage<TransformedCallResponse> {
    fn text(&self) -> String {
        let response_raw_values = self
            .command_response
            .response_raw
            .iter()
            .map(|felt| felt.into_hex_string())
            .collect::<Vec<_>>()
            .join(", ");

        styling::OutputBuilder::new()
            .success_message("Call completed")
            .blank_line()
            .field("Response", &self.command_response.response)
            .field("Response Raw", &format!("[{response_raw_values}]"))
            .build()
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
