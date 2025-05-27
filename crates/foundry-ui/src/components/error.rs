use serde::Serialize;

use crate::Message;

use super::tagged::TaggedMessage;

/// Warning textual message.
#[derive(Serialize)]
pub struct ErrorMessage<'a> {
    text: &'a str,
}

impl<'a> ErrorMessage<'a> {
    #[must_use]
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl Message for ErrorMessage<'_> {
    fn text(&self) -> String {
        let tagged_message = TaggedMessage::styled("ERROR", self.text, "red");
        tagged_message.text()
    }
}
