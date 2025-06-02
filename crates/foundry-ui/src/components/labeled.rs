use serde::Serialize;

use crate::Message;

/// Generic message with `label` prefix.
///
/// The label prefix can be stylized in text mode.
/// e.g. "Tests: 1 passed, 1 failed"
#[derive(Serialize)]
pub struct LabeledMessage<'a, T> {
    message_type: &'a str,
    label: &'a str,
    message: &'a T,
}

impl<'a, T> LabeledMessage<'a, T> {
    #[must_use]
    pub fn new(label: &'a str, message: &'a T) -> Self {
        Self {
            message_type: "labeled",
            label,
            message,
        }
    }
}

impl<T: Message> Message for LabeledMessage<'_, T> {
    fn text(&self) -> String {
        format!("{}: {}", self.label, self.message.text())
    }

    fn json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize message to JSON")
    }
}
