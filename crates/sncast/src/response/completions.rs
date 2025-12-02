use foundry_ui::Message;
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Serialize)]
pub struct CompletionsMessage {
    pub completions: String,
}

impl Message for CompletionsMessage {
    fn text(&self) -> String {
        self.completions.clone()
    }

    fn json(&self) -> Value {
        json!(self)
    }
}
