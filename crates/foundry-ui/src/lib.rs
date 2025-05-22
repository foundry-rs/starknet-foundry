use components::TaggedMessage;
use formats::{NumbersFormat, OutputFormat};
pub use message::*;

pub mod components;
pub mod formats;
pub mod message;

/// An abstraction around console output which stores preferences for output format (human vs JSON),
/// colour, etc.
///
/// All human-oriented messaging (basically all writes to `stdout`) must go through this object.
#[derive(Debug, Default, Clone)]
pub struct Ui {
    output_format: OutputFormat,
    numbers_format: NumbersFormat,
}

impl Ui {
    /// Create a new [`Ui`] instance configured with the given output format and numbers format.
    #[must_use]
    pub fn new(output_format: OutputFormat, numbers_format: NumbersFormat) -> Self {
        Self {
            output_format,
            numbers_format,
        }
    }

    /// Get the output format of this [`Ui`] instance.
    #[must_use]
    pub fn output_format(&self) -> OutputFormat {
        self.output_format
    }

    /// Get the numbers format of this [`Ui`] instance.
    #[must_use]
    pub fn numbers_format(&self) -> NumbersFormat {
        self.numbers_format
    }

    pub fn print<T>(&self, message: &T)
    where
        T: Message + serde::Serialize,
    {
        match self.output_format {
            OutputFormat::Human => message.print_human(self.numbers_format, false),
            OutputFormat::Json => message.print_json(self.numbers_format, false),
        }
    }

    pub fn print_as_err<T>(&self, message: &T)
    where
        T: Message + serde::Serialize,
    {
        match self.output_format {
            OutputFormat::Human => message.print_human(self.numbers_format, true),
            OutputFormat::Json => message.print_json(self.numbers_format, true),
        }
    }

    pub fn print_warning(&self, text: &str) {
        self.print(&TaggedMessage::styled("WARNING", text, "yellow"));
    }

    pub fn print_error(&self, text: &str) {
        self.print(&TaggedMessage::styled("ERROR", text, "red"));
    }
}
