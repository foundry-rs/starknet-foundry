use super::command::CommandResponse;
use crate::response::cast_message::SncastMessage;
use crate::response::cast_message::SncastTextMessage;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct VerifyResponse {
    pub message: String,
}

impl CommandResponse for VerifyResponse {}

impl SncastTextMessage for SncastMessage<VerifyResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Verification completed")
            .blank_line()
            .text_field(&self.command_response.message)
            .build()
    }
}
