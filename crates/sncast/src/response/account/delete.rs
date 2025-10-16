use crate::response::cast_message::SncastCommandMessage;
use crate::response::cast_message::SncastMessage;
use crate::response::command::CommandResponse;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct AccountDeleteResponse {
    pub result: String,
}

impl CommandResponse for AccountDeleteResponse {}

impl SncastCommandMessage for SncastMessage<AccountDeleteResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Account deleted")
            .blank_line()
            .text_field(&self.command_response.result)
            .build()
    }
}
