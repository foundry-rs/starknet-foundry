use console::style;
use serde::Serialize;
use serde_json::json;

use crate::Message;

use super::tagged::TaggedMessage;

/// Warning message.
#[derive(Serialize)]
pub struct WarningMessage<'a, T: Message> {
    message: &'a T,
}

impl<'a, T: Message> WarningMessage<'a, T> {
    #[must_use]
    pub fn new(message: &'a T) -> Self {
        Self { message }
    }
}

impl<T: Message> Message for WarningMessage<'_, T> {
    fn text(&self) -> String {
        let tag = style("WARNING").yellow().to_string();
        let tagged_message = TaggedMessage::new(&tag, self.message);
        tagged_message.text()
    }

    fn json(&self) -> String {
        serde_json::to_string(&json!(
            {
                "message_type": "warning",
                "message": self.message.json(),
            }
        ))
        .expect("Failed to serialize as JSON")
    }
}
