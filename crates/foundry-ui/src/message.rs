use serde::Serialize;

use crate::formats::NumbersFormat;

pub trait Message {
    /// Return textual representation of this message.
    ///
    /// Default implementation returns empty string, making [`Ui`] skip printing this message.
    fn text(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized + Serialize,
    {
        let _ = numbers_format;
        String::new()
    }

    fn print_human(&self, numbers_format: NumbersFormat, print_as_err: bool)
    where
        Self: Sized + Serialize,
    {
        let text = self.text(numbers_format);
        if !text.is_empty() && print_as_err {
            eprintln!("{text}");
        } else if !text.is_empty() && !print_as_err {
            println!("{text}");
        }
    }

    fn json(&self, numbers_format: NumbersFormat) -> String
    where
        Self: Sized + Serialize,
    {
        let _ = numbers_format;
        serde_json::to_string(self).unwrap_or_else(|_| "Invalid JSON".to_string())
    }

    fn print_json(&self, numbers_format: NumbersFormat, print_as_err: bool)
    where
        Self: Serialize + Sized,
    {
        let json = self.json(numbers_format);
        if print_as_err {
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
