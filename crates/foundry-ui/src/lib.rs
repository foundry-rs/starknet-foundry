use components::TaggedMessage;
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
/// All human-oriented messaging (basically all writes to `stdout`) must go through this object.
#[derive(Debug)]
pub struct Ui {
    output_format: OutputFormat,
    // TODO: Add state here, that can be used for e.g. spinner
}

impl Ui {
    /// Create a new [`Ui`] instance configured with the given output format and numbers format.
    #[must_use]
    pub fn new(output_format: OutputFormat) -> Self {
        Self { output_format }
    }

    /// Get the output format of this [`Ui`] instance.
    #[must_use]
    pub fn output_format(&self) -> OutputFormat {
        self.output_format
    }

    pub fn print<T>(&self, message: &T)
    where
        T: Message + serde::Serialize,
    {
        match self.output_format {
            OutputFormat::Human => message.print_human(false),
            OutputFormat::Json => message.print_json(false),
        }
    }

    pub fn print_as_err<T>(&self, message: &T)
    where
        T: Message + serde::Serialize,
    {
        match self.output_format {
            OutputFormat::Human => message.print_human(true),
            OutputFormat::Json => message.print_json(true),
        }
    }

    pub fn print_warning(&self, message: &str) {
        self.print(&TaggedMessage::styled(
            "WARNING",
            message.as_ref(),
            "yellow",
        ))
    }
}
