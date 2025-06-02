use serde::Serialize;

use crate::Message;

/// Generic message with `tag` prefix.
///
/// The tag prefix can be stylized in text mode.
/// e.g. "[WARNING]: An example warning message"
#[derive(Serialize)]
pub struct TaggedMessage<'a, T: Message> {
    message_type: &'a str,
    tag: &'a str,
    message: &'a T,
}

impl<'a, T: Message> TaggedMessage<'a, T> {
    #[must_use]
    pub fn new(tag: &'a str, message: &'a T) -> Self {
        Self {
            message_type: "tagged",
            tag,
            message,
        }
    }
}

impl<T: Message> Message for TaggedMessage<'_, T> {
    fn text(&self) -> String {
        format!("[{}] {}", self.tag, self.message.text())
    }

    fn json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize as JSON")
    }
}
