use foundry_ui::Message;
use serde::Serialize;
use serde_json::Value;

use super::command::CommandResponse;
#[derive(Serialize)]
pub struct SncastMessage<T: CommandResponse> {
    pub command: String,
    pub command_response: T,
}

pub trait SncastCommandMessage {
    fn text(&self) -> String;
}

impl<T: CommandResponse> Message for SncastMessage<T>
where
    SncastMessage<T>: SncastCommandMessage,
{
    fn text(&self) -> String {
        SncastCommandMessage::text(self)
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap_or_else(|err| {
            serde_json::json!({
                "error": "Failed to serialize response",
                "command": self.command,
                "details": err.to_string()
            })
        })
    }
}
