use camino::Utf8PathBuf;
use foundry_ui::Message;
use foundry_ui::styling;
use serde::Serialize;
use serde_json::Value;

use crate::response::cast_message::SncastMessage;
use crate::response::command::CommandResponse;

#[derive(Serialize, Clone)]
pub struct MulticallNewResponse {
    pub path: Utf8PathBuf,
    pub content: String,
}

impl CommandResponse for MulticallNewResponse {}

impl Message for SncastMessage<MulticallNewResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .command_title(&self.command)
            .success_message("Multicall template created successfully")
            .blank_line()
            .field("Path", self.command_response.path.as_ref())
            .blank_line()
            .text_field(&self.command_response.content)
            .build()
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap()
    }
}
