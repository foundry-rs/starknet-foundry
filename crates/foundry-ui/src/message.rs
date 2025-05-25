use serde::Serialize;

/// A typed object that can be either printed as a human-readable message or serialized as JSON.
///
/// The [`TypedMessage`][crate::components::TypedMessage] and [`TaggedMessage`][crate::components::TaggedMessage]
/// structs are the most frequently used kinds of messages.
pub trait Message {
    /// Return textual (human) representation of this message.
    ///
    /// Default implementation returns empty string, making [`Ui`] skip printing this message.
    fn text(&self) -> String
    where
        Self: Sized + Serialize,
    {
        String::new()
    }

    /// Return JSON representation of this message.
    fn json(&self) -> String
    where
        Self: Sized + Serialize,
    {
        serde_json::to_string(self).unwrap_or_else(|_| "Invalid JSON".to_string())
    }
}

impl Message for &str {
    fn text(&self) -> String {
        (*self).to_string()
    }
}

impl Message for String {
    fn text(&self) -> String {
        self.to_string()
    }
}
