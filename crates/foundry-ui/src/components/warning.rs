use console::Style;
use serde::Serialize;

use crate::Message;

use super::tagged::TaggedMessage;

/// Warning message.
#[derive(Serialize)]
pub struct WarningMessage<'a, T: Message> {
    message_type: &'a str,
    message: &'a T,
}

impl<'a, T: Message> WarningMessage<'a, T> {
    #[must_use]
    pub fn new(message: &'a T) -> Self {
        Self {
            message_type: "warning",
            message,
        }
    }
}

impl<T: Message + Serialize> Message for WarningMessage<'_, T> {
    fn text(&self) -> String {
        let tag = Style::new().yellow().apply_to("WARNING").to_string();
        let tagged_message = TaggedMessage::new(&tag, self.message);
        tagged_message.text()
    }

    fn json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize message to JSON")
    }
}
