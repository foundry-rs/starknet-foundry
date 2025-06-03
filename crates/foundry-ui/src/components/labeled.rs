use serde::Serialize;
use serde_json::json;

use crate::Message;

/// Generic message with `label` prefix.
///
/// e.g. "Tests: 1 passed, 1 failed"
#[derive(Serialize)]
pub struct LabeledMessage<'a, T: Message> {
    label: &'a str,
    message: &'a T,
}

impl<'a, T: Message> LabeledMessage<'a, T> {
    #[must_use]
    pub fn new(label: &'a str, message: &'a T) -> Self {
        Self { label, message }
    }
}

impl<T: Message> Message for LabeledMessage<'_, T> {
    fn text(&self) -> String {
        format!("{}: {}", self.label, self.message.text())
    }

    fn json(&self) -> String {
        serde_json::to_string(&json!(
            {
                "message_type": "labeled",
                "label": self.label,
                "message": self.message.json(),
            }
        ))
        .expect("Failed to serialize as JSON")
    }
}
