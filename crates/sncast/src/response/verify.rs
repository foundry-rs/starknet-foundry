use super::command::CommandResponse;
use crate::response::cast_message::SncastMessage;
use crate::response::helpers::serialize_json;
use foundry_ui::Message;
use foundry_ui::styling;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Clone)]
pub struct VerifyResponse {
    pub message: String,
}

impl CommandResponse for VerifyResponse {}

impl Message for SncastMessage<VerifyResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Verification completed")
            .blank_line()
            .text_field(&self.command_response.message)
            .build()
    }

    fn json(&self) -> Value {
        serialize_json(self)
    }
}
