use serde::Serialize;

/// A typed object that can be either printed as a human-readable message or serialized as JSON.
pub trait Message: Serialize {
    /// Return textual (human) representation of this message.
    ///
    /// Default implementation returns empty string, making [`Ui`] skip printing this message.
    fn text(&self) -> String;

    /// Return JSON representation of this message.
    fn json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "Invalid JSON".to_string())
    }
}

impl<T: ToString> Message for T
where
    T: Serialize,
{
    fn text(&self) -> String {
        self.to_string()
    }
}
