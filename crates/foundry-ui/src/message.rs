use serde::Serialize;

/// A typed object that can be either printed as a human-readable message or serialized as JSON.
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

    /// Print textual (human) representation of this message.
    fn print_human(&self, print_as_err: bool)
    where
        Self: Sized + Serialize,
    {
        let text = self.text();
        if !text.is_empty() && print_as_err {
            eprintln!("{text}");
        } else if !text.is_empty() && !print_as_err {
            println!("{text}");
        }
    }

    /// Return JSON representation of this message.
    fn json(&self) -> String
    where
        Self: Sized + Serialize,
    {
        serde_json::to_string(self).unwrap_or_else(|_| "Invalid JSON".to_string())
    }

    /// Print JSON representation of this message.
    fn print_json(&self, print_as_err: bool)
    where
        Self: Serialize + Sized,
    {
        let json = self.json();
        if print_as_err {
            eprintln!("{json}");
        } else {
            println!("{json}");
        }
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
