use crate::response::cast_message::SncastCommandMessage;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct VerifyResponse {
    pub message: String,
}

impl SncastCommandMessage for VerifyResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Verification completed")
            .blank_line()
            .text_field(&self.message)
            .build()
    }
}
