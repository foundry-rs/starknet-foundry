use serde::Serialize;
use serde_json::{Value, json};

use crate::Message;

/// Generic message with `tag` prefix.
///
/// e.g. "[WARNING]: An example warning message"
#[derive(Serialize)]
pub struct TaggedMessage<'a, T: Message> {
    tag: &'a str,
    message: &'a T,
}

impl<'a, T: Message> TaggedMessage<'a, T> {
    #[must_use]
    pub fn new(tag: &'a str, message: &'a T) -> Self {
        Self { tag, message }
    }
}

impl<T: Message> Message for TaggedMessage<'_, T> {
    fn text(&self) -> String {
        format!("[{}] {}", self.tag, self.message.text())
    }

    fn json(&self) -> Value {
        json!(
            {
                "message_type": "tagged",
                "tag": self.tag,
                "message": self.message.json(),
            }
        )
    }
}
