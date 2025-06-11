use foundry_ui::Message;
use foundry_ui::styling;
use serde::Serialize;
use serde_json::Value;

use crate::response::cast_message::SncastMessage;
use crate::response::command::CommandResponse;

#[derive(Serialize, Debug, Clone)]
pub struct ScriptRunResponse {
    pub status: String,
    pub message: Option<String>,
}

impl CommandResponse for ScriptRunResponse {}

impl Message for SncastMessage<ScriptRunResponse> {
    fn text(&self) -> String {
        let mut builder = styling::OutputBuilder::new()
            .success_message("Script execution completed")
            .blank_line()
            .field("Status", &self.command_response.status);

        if let Some(message) = &self.command_response.message {
            builder = builder.blank_line().text_field(message);
        }

        builder.build()
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap()
    }
}
