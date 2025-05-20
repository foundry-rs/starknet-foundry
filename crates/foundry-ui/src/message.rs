use serde::Serialize;

use crate::formats::NumbersFormat;

pub trait Message {
    /// Return textual representation of this message.
    ///
    /// Default implementation returns json string, making [`Ui`] skip printing this message.
    fn text(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized + Serialize,
    {
        let _ = numbers_format;
        serde_json::to_string(&self).unwrap_or_else(|_| "Invalid JSON".to_string())
    }

    fn print_human(&self, numbers_format: NumbersFormat, is_err: bool)
    where
        Self: Sized + Serialize,
    {
        let text = self.text(numbers_format);
        if !text.is_empty() && is_err {
            eprintln!("{text}");
        } else if !text.is_empty() && !is_err {
            println!("{text}");
        }
    }

    fn print_json(&self, is_err: bool)
    where
        Self: Serialize,
    {
        let json = serde_json::to_string(self).unwrap_or_else(|_| "Invalid JSON".to_string());
        if is_err {
            eprintln!("{json}");
        } else {
            println!("{json}");
        }
    }
}

impl Message for &str {
    fn text(&self, numbers_format: NumbersFormat) -> String {
        let _ = numbers_format;
        (*self).to_string()
    }
}

impl Message for String {
    fn text(&self, numbers_format: NumbersFormat) -> String {
        let _ = numbers_format;
        self.to_string()
    }
}
