use formats::{NumbersFormat, OutputFormat};
pub use message::*;

pub mod formats;
pub mod message;

/// An abstraction around console output which stores preferences for output format (human vs JSON),
/// colour, etc.
///
/// All human-oriented messaging (basically all writes to `stdout`) must go through this object.
#[derive(Debug)]
pub struct Ui {}

impl Ui {
    pub fn print<T>(message: &T, output_format: OutputFormat, numbers_format: NumbersFormat)
    where
        T: Message + serde::Serialize,
    {
        match output_format {
            OutputFormat::Human => message.print_human(numbers_format, false),
            OutputFormat::Json => message.print_json(false),
        }
    }

    pub fn print_err<T>(message: &T, output_format: OutputFormat, numbers_format: NumbersFormat)
    where
        T: Message + serde::Serialize,
    {
        match output_format {
            OutputFormat::Human => message.print_human(numbers_format, true),
            OutputFormat::Json => message.print_json(true),
        }
    }
}
