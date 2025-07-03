use crate::response::cast_message::SncastMessage;
use crate::response::command::CommandResponse;
use foundry_ui::Message;
use foundry_ui::styling;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;

#[derive(Serialize, Clone)]
pub struct AccountDeleteResponse {
    pub result: String,
}

impl CommandResponse for AccountDeleteResponse {}

impl Message for SncastMessage<AccountDeleteResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Account deleted")
            .blank_line()
            .text_field(&self.command_response.result)
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
