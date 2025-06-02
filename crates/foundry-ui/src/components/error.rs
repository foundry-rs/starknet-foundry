use anyhow::Error;
use console::Style;
use serde::Serialize;

use crate::Message;

use super::tagged::TaggedMessage;

/// Error message.
#[derive(Serialize)]
pub struct ErrorMessage<T: Message> {
    message_type: &'static str,
    message: T,
}

impl<T: Message> ErrorMessage<T> {
    #[must_use]
    pub fn new(message: T) -> Self {
        Self {
            message_type: "error",
            message,
        }
    }
}

impl<T: Message> Message for ErrorMessage<T> {
    fn text(&self) -> String {
        let tag = Style::new().red().apply_to("ERROR").to_string();
        let tagged_message = TaggedMessage::new(&tag, &self.message);
        tagged_message.text()
    }

    fn json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize as JSON")
    }
}

impl From<Error> for ErrorMessage<String> {
    fn from(error: Error) -> Self {
        Self::new(format!("{error:#}"))
    }
}
