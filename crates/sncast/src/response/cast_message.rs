use serde::Serialize;

use super::command::CommandResponse;
#[derive(Serialize)]
pub struct SncastMessage<T: CommandResponse> {
    pub command: String,
    pub command_response: T,
}

impl<T: CommandResponse> SncastMessage<T> {
    pub fn to_json(&self) -> serde_json::Value {
        match serde_json::to_value(&self.command_response) {
            Ok(serde_json::Value::Object(mut map)) => {
                map.insert(
                    "command".to_string(),
                    serde_json::Value::String(self.command.clone()),
                );
                serde_json::Value::Object(map)
            }
            Ok(other) => {
                serde_json::json!({
                    "error": "Expected a map for `command_response`",
                    "command": self.command,
                    "details": other.to_string()
                })
            }
            Err(err) => {
                serde_json::json!({
                    "error": "Failed to serialize response",
                    "command": self.command,
                    "details": err.to_string()
                })
            }
        }
    }
}
