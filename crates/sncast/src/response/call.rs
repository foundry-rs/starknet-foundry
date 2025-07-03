use super::command::CommandResponse;
use crate::response::cast_message::SncastMessage;
use conversions::serde::serialize::CairoSerialize;
use conversions::string::IntoHexStr;
use foundry_ui::Message;
use foundry_ui::styling;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use starknet_types_core::felt::Felt;

#[derive(Serialize, CairoSerialize, Clone)]
pub struct CallResponse {
    pub response: Vec<Felt>,
}

impl CommandResponse for CallResponse {}

impl Message for SncastMessage<CallResponse> {
    fn text(&self) -> String {
        let response_values = self
            .command_response
            .response
            .iter()
            .map(|felt| felt.into_hex_string())
            .collect::<Vec<_>>()
            .join(", ");

        styling::OutputBuilder::new()
            .success_message("Call completed")
            .blank_line()
            .field("Response", &format!("[{response_values}]"))
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
