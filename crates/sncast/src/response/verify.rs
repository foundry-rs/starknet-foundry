use crate::response::cast_message::SncastCommandMessage;
use crate::response::cast_message::SncastMessage;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct VerifyResponse {
    pub message: String,
}

impl SncastCommandMessage for SncastMessage<VerifyResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Verification completed")
            .blank_line()
            .text_field(&self.command_response.message)
            .build()
    }
}
