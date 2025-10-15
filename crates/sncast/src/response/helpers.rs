use crate::response::{cast_message::SncastMessage, command::CommandResponse};

pub fn serialize_json(sncast_message: &SncastMessage<impl CommandResponse>) -> serde_json::Value {
    serde_json::to_value(&sncast_message.command_response).unwrap_or_else(|err| {
        serde_json::json!({
            "error": "Failed to serialize response",
            "command": sncast_message.command,
            "details": err.to_string()
        })
    })
}
