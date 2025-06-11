pub use message::*;

pub mod components;
pub mod message;
pub mod styling;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Human,
    Json,
}

/// An abstraction around console output which stores preferences for output format (human vs JSON),
/// colour, etc.
///
/// All messaging (basically all writes to `stdout`) must go through this object.
#[derive(Debug, Default)]
pub struct UI {
    output_format: OutputFormat,
    // TODO(#3395): Add state here, that can be used for spinner
}

impl UI {
    /// Create a new [`UI`] instance configured with the given output format.
    #[must_use]
    pub fn new(output_format: OutputFormat) -> Self {
        Self { output_format }
    }

    /// Create a [`String`] representation of the given message based on the configured output format.
    fn format_message(&self, message: &impl Message) -> String {
        match self.output_format {
            OutputFormat::Human => message.text(),
            OutputFormat::Json => message.json().to_string(),
        }
    }

    /// Print the given message to stdout using the configured output format.
    pub fn println(&self, message: &impl Message) {
        println!("{}", self.format_message(message));
    }

    /// Print the given message to stderr using the configured output format.
    pub fn eprintln(&self, message: &impl Message) {
        eprintln!("{}", self.format_message(message));
    }
}
