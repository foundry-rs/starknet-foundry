use formats::{NumbersFormat, OutputFormat};
pub use message::*;

pub mod formats;
pub mod message;

/// An abstraction around console output which stores preferences for output format (human vs JSON),
/// colour, etc.
///
/// All human-oriented messaging (basically all writes to `stdout`) must go through this object.
#[derive(Debug)]
pub struct Ui {
    output_format: OutputFormat,
    numbers_format: NumbersFormat,
}

impl Ui {
    /// Create a new [`Ui`] instance configured with the given output format and numbers format.
    pub fn new(output_format: OutputFormat, numbers_format: NumbersFormat) -> Self {
        Self {
            output_format,
            numbers_format,
        }
    }

    /// Get the output format of this [`Ui`] instance.
    pub fn output_format(&self) -> OutputFormat {
        self.output_format
    }

    /// Get the numbers format of this [`Ui`] instance.
    pub fn numbers_format(&self) -> NumbersFormat {
        self.numbers_format
    }

    pub fn print<T: Message>(&self, message: &T)
    where
        T: Message + serde::Serialize,
    {
        match self.output_format {
            OutputFormat::Human => message.print_human(self.numbers_format, false),
            OutputFormat::Json => message.print_json(self.numbers_format, false),
        }
    }

    pub fn print_as_err<T: Message>(&self, message: &T)
    where
        T: Message + serde::Serialize,
    {
        match self.output_format {
            OutputFormat::Human => message.print_human(self.numbers_format, true),
            OutputFormat::Json => message.print_json(self.numbers_format, true),
        }
    }
}
