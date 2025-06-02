use console::style;
use serde::Serialize;
use serde_json::json;

use crate::Message;

use super::tagged::TaggedMessage;

/// Error message.
#[derive(Serialize)]
pub struct ErrorMessage<'a, T: Message> {
    message: &'a T,
}

impl<'a, T: Message> ErrorMessage<'a, T> {
    #[must_use]
    pub fn new(message: &'a T) -> Self {
        Self { message }
    }
}

impl<T: Message> Message for ErrorMessage<'_, T> {
    fn text(&self) -> String {
        let tag = style("ERROR").red().to_string();
        let tagged_message = TaggedMessage::new(&tag, self.message);
        tagged_message.text()
    }

    fn json(&self) -> String {
        serde_json::to_string(&json!({
            "message_type": "error",
            "message": self.message.json(),
        }))
        .expect("Failed to serialize as JSON")
    }
}
