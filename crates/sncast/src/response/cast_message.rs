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
                let mut map = serde_json::Map::new();
                map.insert(
                    "command".to_string(),
                    serde_json::Value::String(self.command.clone()),
                );
                map.insert(
                    "error".to_string(),
                    serde_json::Value::String(
                        "expected a JSON object/map for command_response".to_string(),
                    ),
                );
                map.insert("command_response".to_string(), other);
                serde_json::Value::Object(map)
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
