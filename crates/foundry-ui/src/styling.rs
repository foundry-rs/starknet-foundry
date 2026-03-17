use console::style;
use starknet_types_core::felt::Felt;
use std::fmt::Write;

#[derive(Debug, Clone)]
enum OutputEntry {
    SuccessMessage(String),
    ErrorMessage(String),
    Field { label: String, value: String },
    BlankLine,
    Text(String),
}

pub struct OutputBuilder {
    entries: Vec<OutputEntry>,
}

impl Default for OutputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn calculate_field_label_width(&self) -> usize {
        self.entries
            .iter()
            .filter_map(|entry| {
                if let OutputEntry::Field { label, .. } = entry {
                    Some(label.len() + 1)
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(0)
    }

    #[must_use]
    pub fn success_message(mut self, message: &str) -> Self {
        self.entries
            .push(OutputEntry::SuccessMessage(message.to_string()));
        self
    }

    #[must_use]
    pub fn error_message(mut self, message: &str) -> Self {
        self.entries
            .push(OutputEntry::ErrorMessage(message.to_string()));
        self
    }

    #[must_use]
    pub fn field(mut self, label_text: &str, value: &str) -> Self {
        self.entries.push(OutputEntry::Field {
            label: label_text.to_string(),
            value: value.to_string(),
        });
        self
    }

    #[must_use]
    pub fn blank_line(mut self) -> Self {
        self.entries.push(OutputEntry::BlankLine);
        self
    }

    #[must_use]
    pub fn text_field(mut self, text: &str) -> Self {
        self.entries.push(OutputEntry::Text(text.to_string()));
        self
    }

    #[must_use]
    pub fn if_some<F, T>(mut self, option: Option<&T>, f: F) -> Self
    where
        F: FnOnce(Self, &T) -> Self,
    {
        if let Some(value) = option {
            self = f(self, value);
        }
        self
    }

    #[must_use]
    pub fn padded_felt_field(self, label: &str, felt: &Felt) -> Self {
        self.field(label, &felt.to_fixed_hex_string())
    }

    #[must_use]
    pub fn felt_field(self, label: &str, felt: &Felt) -> Self {
        self.field(label, &felt.to_hex_string())
    }

    #[must_use]
    pub fn felt_list_field(self, label: &str, felts: &[Felt]) -> Self {
        let felts = felts.iter().map(Felt::to_hex_string).collect::<Vec<_>>();
        self.field(label, &format!("[{}]", felts.join(", ")))
    }

    #[must_use]
    pub fn build(self) -> String {
        let field_width = self.calculate_field_label_width();
        let mut content = String::new();

        for entry in self.entries {
            match entry {
                OutputEntry::SuccessMessage(message) => {
                    writeln!(content, "{}: {}", style("Success").green(), message).unwrap();
                }
                OutputEntry::ErrorMessage(message) => {
                    writeln!(content, "{}: {}", style("Error").red(), message).unwrap();
                }
                OutputEntry::Field { label, value } => {
                    writeln!(
                        content,
                        "{:field_width$} {}",
                        format!("{}:", label),
                        style(value).yellow(),
                    )
                    .unwrap();
                }
                OutputEntry::BlankLine => {
                    content.push('\n');
                }
                OutputEntry::Text(text) => {
                    if !content.is_empty() && !content.ends_with('\n') {
                        content.push('\n');
                    }
                    content.push_str(&text);
                    content.push('\n');
                }
            }
        }

        if content.ends_with('\n') {
            content.pop();
        }
        content
    }
}
