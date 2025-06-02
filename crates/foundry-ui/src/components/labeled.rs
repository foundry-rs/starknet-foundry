use serde::Serialize;

use crate::Message;

/// Generic textual message with `ty` prefix.
///
/// The type prefix can be stylized in text mode.
/// e.g. "Tests: 1 passed, 1 failed"
#[derive(Serialize)]
pub struct LabeledMessage<'a> {
    message_type: &'a str,
    label: &'a str,
    text: &'a str,
}

impl<'a> LabeledMessage<'a> {
    #[must_use]
    pub fn new(label: &'a str, text: &'a str) -> Self {
        Self {
            message_type: "labeled",
            label,
            text,
        }
    }
}

impl Message for LabeledMessage<'_> {
    fn text(&self) -> String {
        format!("{}: {}", self.label, self.text)
    }

    fn json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize message to JSON")
    }
}
