use serde::Serialize;

use crate::Message;

use super::tagged::TaggedMessage;

/// Warning textual message.
#[derive(Serialize)]
pub struct WarningMessage<'a> {
    text: &'a str,
}

impl<'a> WarningMessage<'a> {
    #[must_use]
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl Message for WarningMessage<'_> {
    fn text(&self) -> String {
        let tagged_message = TaggedMessage::styled("WARNING", self.text, "yellow");
        tagged_message.text()
    }
}
