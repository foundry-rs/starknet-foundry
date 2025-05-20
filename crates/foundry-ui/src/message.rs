use serde::Serialize;
use serde_json;

pub trait Message {
    /// Return textual representation of this message.
    ///
    /// Default implementation returns json string, making [`Ui`] skip printing this message.
    fn text(self) -> String
    where
        Self: Sized + Serialize,
    {
        serde_json::to_string(&self).unwrap_or_else(|_| "Invalid JSON".to_string())
    }

    fn print_human(self)
    where
        Self: Sized + Serialize,
    {
        let text = self.text();
        if !text.is_empty() {
            println!("{text}");
        }
    }

    fn print_json(&self)
    where
        Self: Serialize,
    {
        let json = serde_json::to_string(self).unwrap_or_else(|_| "Invalid JSON".to_string());
        println!("{json}");
    }
}

impl Message for &str {
    fn text(self) -> String {
        self.to_string()
    }
}

impl Message for String {
    fn text(self) -> String {
        self
    }
}
