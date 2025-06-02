use console::Style;
use serde::Serialize;

use crate::Message;

use super::tagged::TaggedMessage;

/// Warning textual message.
#[derive(Serialize)]
pub struct ErrorMessage<'a> {
    message_type: &'a str,
    text: &'a str,
}

impl<'a> ErrorMessage<'a> {
    #[must_use]
    pub fn new(text: &'a str) -> Self {
        Self {
            message_type: "error",
            text,
        }
    }
}

impl Message for ErrorMessage<'_> {
    fn text(&self) -> String {
        let tag = Style::new().red().apply_to("ERROR").to_string();
        let tagged_message = TaggedMessage::new(&tag, self.text);
        tagged_message.text()
    }

    fn json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize message to JSON")
    }
}
