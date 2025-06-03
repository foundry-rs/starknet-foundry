use anyhow::Error;
use console::style;
use serde::Serialize;
use serde_json::{Value, json};

use crate::Message;

use super::tagged::TaggedMessage;

/// Error message.
#[derive(Serialize)]
pub struct ErrorMessage<T: Message>(T);

impl<T: Message> ErrorMessage<T> {
    #[must_use]
    pub fn new(message: T) -> Self {
        Self(message)
    }
}

impl<T: Message> Message for ErrorMessage<T> {
    fn text(&self) -> String {
        let tag = style("ERROR").red().to_string();
        let tagged_message = TaggedMessage::new(&tag, &self.0);
        tagged_message.text()
    }

    fn json(&self) -> Value {
        json!({
            "message_type": "error",
            "message": self.0.json(),
        })
    }
}

impl From<Error> for ErrorMessage<String> {
    fn from(error: Error) -> Self {
        Self::new(format!("{error:#}"))
    }
}
