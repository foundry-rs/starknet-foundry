use crate::response::cast_message::SncastCommandMessage;
use crate::response::cast_message::SncastMessage;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct ScriptRunResponse {
    pub status: String,
    pub message: Option<String>,
}

impl SncastCommandMessage for SncastMessage<ScriptRunResponse> {
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
}
