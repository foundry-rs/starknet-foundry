use console::style;
use serde::Serialize;
use serde_json::json;

use crate::Message;

use super::tagged::TaggedMessage;

/// Warning message.
#[derive(Serialize)]
pub struct WarningMessage<T: Message>(T);

impl<T: Message> WarningMessage<T> {
    #[must_use]
    pub fn new(message: T) -> Self {
        Self(message)
    }
}

impl<T: Message> Message for WarningMessage<T> {
    fn text(&self) -> String {
        let tag = style("WARNING").yellow().to_string();
        let tagged_message = TaggedMessage::new(&tag, &self.0);
        tagged_message.text()
    }

    fn json(&self) -> String {
        serde_json::to_string(&json!({
            "message_type": "warning",
            "message": self.0.json(),
        }))
        .expect("Failed to serialize as JSON")
    }
}
