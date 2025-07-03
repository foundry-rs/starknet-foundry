use crate::response::cast_message::SncastMessage;
use crate::response::command::CommandResponse;
use foundry_ui::Message;
use foundry_ui::styling;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;

#[derive(Serialize, Clone)]
pub struct ScriptInitResponse {
    pub message: String,
}

impl CommandResponse for ScriptInitResponse {}

impl Message for SncastMessage<ScriptInitResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Script initialization completed")
            .blank_line()
            .text_field(&self.command_response.message)
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
