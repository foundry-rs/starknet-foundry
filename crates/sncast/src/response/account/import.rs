use crate::response::cast_message::SncastMessage;
use crate::response::command::CommandResponse;
use foundry_ui::Message;
use foundry_ui::styling;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;

#[derive(Serialize, Clone)]
pub struct AccountImportResponse {
    pub add_profile: Option<String>,
    pub account_name: String,
}

impl CommandResponse for AccountImportResponse {}

impl Message for SncastMessage<AccountImportResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Account imported successfully")
            .blank_line()
            .field("Account Name", &self.command_response.account_name)
            .if_some(
                self.command_response.add_profile.as_ref(),
                |builder, profile| builder.field("Add Profile", profile),
            )
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
