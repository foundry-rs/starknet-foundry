use crate::response::cast_message::SncastCommandMessage;
use crate::response::cast_message::SncastMessage;
use crate::response::command::CommandResponse;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct ScriptInitResponse {
    pub message: String,
}

impl CommandResponse for ScriptInitResponse {}

impl SncastCommandMessage for SncastMessage<ScriptInitResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Script initialization completed")
            .blank_line()
            .text_field(&self.command_response.message)
            .build()
    }
}
