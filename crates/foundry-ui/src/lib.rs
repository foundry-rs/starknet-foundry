pub use message::*;

pub mod components;
pub mod message;

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
pub trait Printer {
    /// Print the given message to stdout using the configured output format.
    fn println(&self, message: &impl Message);

    /// Print the given message to stderr using the configured output format.
    fn eprintln(&self, message: &impl Message);
}

#[derive(Debug, Default, Clone)]
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

    // TODO: Will be removed in 3022-3-use-foundry-ui-in-sncast
    /// Get the output format of this [`UI`] instance.
    #[must_use]
    pub fn output_format(&self) -> OutputFormat {
        self.output_format
    }

    /// Create a [`String`] representation of the given message based on the configured output format.
    fn format_message(&self, message: &impl Message) -> String {
        match self.output_format {
            OutputFormat::Human => message.text(),
            OutputFormat::Json => message.json().to_string(),
        }
    }
}
impl Printer for UI {
    fn println(&self, message: &impl Message) {
        println!("{}", self.format_message(message));
    }

    fn eprintln(&self, message: &impl Message) {
        eprintln!("{}", self.format_message(message));
    }
}
