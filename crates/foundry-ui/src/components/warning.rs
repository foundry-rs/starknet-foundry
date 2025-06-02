use console::Style;
use serde::Serialize;

use crate::Message;

use super::tagged::TaggedMessage;

/// Warning textual message.
#[derive(Serialize)]
pub struct WarningMessage<'a> {
    message_type: &'a str,
    text: &'a str,
}

impl<'a> WarningMessage<'a> {
    #[must_use]
    pub fn new(text: &'a str) -> Self {
        Self {
            message_type: "warning",
            text,
        }
    }
}

impl Message for WarningMessage<'_> {
    fn text(&self) -> String {
        let tag = Style::new().yellow().apply_to("WARNING").to_string();
        let tagged_message = TaggedMessage::new(&tag, self.text);
        tagged_message.text()
    }

    fn json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize message to JSON")
    }
}
