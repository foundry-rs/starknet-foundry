use crate::response::cast_message::SncastCommandMessage;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct AccountDeleteResponse {
    pub result: String,
}

impl SncastCommandMessage for AccountDeleteResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Account deleted")
            .blank_line()
            .text_field(&self.result)
            .build()
    }
}
