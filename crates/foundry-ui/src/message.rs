use serde::Serialize;

/// A typed object that can be either printed as a human-readable message or serialized as JSON.
pub trait Message: Serialize {
    /// Return textual (human) representation of this message.
    fn text(&self) -> String;

    /// Return JSON representation of this message.
    fn json(&self) -> String;
}

impl<T: ToString> Message for T
where
    T: Serialize,
{
    fn text(&self) -> String {
        self.to_string()
    }

    fn json(&self) -> String {
        serde_json::to_string(&serde_json::json!({ "message": self.to_string() }))
            .expect("Failed to serialize message to JSON")
    }
}
