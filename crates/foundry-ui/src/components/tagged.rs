use serde::Serialize;

use crate::Message;

/// Generic textual message with `tag` prefix.
///
/// The tag prefix can be stylized in text mode.
/// e.g. "[WARNING]: An example warning message"
#[derive(Serialize)]
pub struct TaggedMessage<'a> {
    message_type: &'a str,
    tag: &'a str,
    text: &'a str,
}

impl<'a> TaggedMessage<'a> {
    #[must_use]
    pub fn new(tag: &'a str, text: &'a str) -> Self {
        Self {
            message_type: "tagged",
            tag,
            text,
        }
    }
}

impl Message for TaggedMessage<'_> {
    fn text(&self) -> String {
        format!("[{}] {}", self.tag, self.text)
    }

    fn json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize message to JSON")
    }
}
