pub use message::*;

pub mod components;
pub mod message;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Human,
    Json,
}

impl OutputFormat {
    #[must_use]
    pub fn from_flag(json: bool) -> Self {
        if json {
            OutputFormat::Json
        } else {
            OutputFormat::Human
        }
    }
}

/// An abstraction around console output which stores preferences for output format (human vs JSON),
/// colour, etc.
///
/// All messaging (basically all writes to `stdout`) must go through this object.
#[derive(Debug, Default, Clone)]
pub struct UI {
    output_format: OutputFormat,
    // TODO(3395): Add state here, that can be used for spinner
}

impl UI {
    /// Create a new [`UI`] instance configured with the given output format.
    #[must_use]
    pub fn new(output_format: OutputFormat) -> Self {
        Self { output_format }
    }

    /// Print the given message to stdout using the configured output format.
    pub fn println<T>(&self, message: &T)
    where
        T: Message + serde::Serialize,
    {
        match self.output_format {
            OutputFormat::Human => println!("{}", message.text()),
            OutputFormat::Json => println!("{}", message.json()),
        }
    }

    /// Print the given message to stderr using the configured output format.
    pub fn eprintln<T>(&self, message: &T)
    where
        T: Message + serde::Serialize,
    {
        match self.output_format {
            OutputFormat::Human => eprintln!("{}", message.text()),
            OutputFormat::Json => eprintln!("{}", message.json()),
        }
    }
}
