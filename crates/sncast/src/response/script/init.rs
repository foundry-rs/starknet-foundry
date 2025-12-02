use crate::response::cast_message::SncastCommandMessage;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct ScriptInitResponse {
    pub message: String,
}

impl SncastCommandMessage for ScriptInitResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Script initialization completed")
            .blank_line()
            .text_field(&self.message)
            .build()
    }
}
