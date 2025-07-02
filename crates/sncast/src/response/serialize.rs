use super::command::CommandResponse;
use crate::response::cast_message::SncastMessage;
use conversions::serde::serialize::CairoSerialize;
use foundry_ui::Message;
use foundry_ui::styling;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use starknet_types_core::felt::Felt;

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct SerializeResponse {
    pub calldata: Vec<Felt>,
}

impl CommandResponse for SerializeResponse {}

impl Message for SncastMessage<SerializeResponse> {
    fn text(&self) -> String {
        let calldata = format!("{:?}", &self.command_response.calldata);

        styling::OutputBuilder::new()
            .field("Calldata", &calldata)
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
