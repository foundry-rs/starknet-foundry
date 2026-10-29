use console::style;
use starknet_types_core::felt::Felt;
use std::fmt::Write;

#[derive(Debug, Clone)]
enum OutputEntry {
    SuccessMessage(String),
    ErrorMessage(String),
    Field {
        label: String,
        value: String,
        extra: Option<String>,
        indent: usize,
    },
    BlankLine,
    Text(String),
    Heading {
        text: String,
        indent: usize,
    },
}

pub struct OutputBuilder {
    entries: Vec<OutputEntry>,
    current_indent: usize,
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
            current_indent: 0,
        }
    }

    fn calculate_field_label_width(&self) -> usize {
        self.entries
            .iter()
            .filter_map(|entry| {
                if let OutputEntry::Field { label, indent, .. } = entry {
                    Some(indent + label.len() + 1)
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(0)
    }

    /// Runs `f` with the indentation increased by `by` spaces relative to the current one.
    /// The previous indentation is restored afterwards.
    #[must_use]
    pub fn indented<F>(mut self, by: usize, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        let previous = self.current_indent;
        self.current_indent = previous + by;
        let mut builder = f(self);
        builder.current_indent = previous;
        builder
    }

    /// Adds a bold line at the current indentation.
    /// Headings are not aligned with fields, so they never widen the value column.
    #[must_use]
    pub fn heading(mut self, text: &str) -> Self {
        self.entries.push(OutputEntry::Heading {
            text: text.to_string(),
            indent: self.current_indent,
        });
        self
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
            extra: None,
            indent: self.current_indent,
        });
        self
    }

    #[must_use]
    pub fn extra(mut self, extra: &str) -> Self {
        if let Some(OutputEntry::Field {
            extra: field_extra, ..
        }) = self.entries.last_mut()
        {
            *field_extra = Some(extra.to_string());
        }
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
                OutputEntry::Field {
                    label,
                    value,
                    extra,
                    indent,
                } => {
                    let mut value_lines = value.lines();
                    let first_line = value_lines.next().unwrap_or_default();
                    let styled_value = style(first_line).yellow();
                    let field_value = match extra {
                        Some(extra) => format!("{styled_value} {extra}"),
                        None => format!("{styled_value}"),
                    };
                    writeln!(
                        content,
                        "{:field_width$} {}",
                        format!("{}{}:", " ".repeat(indent), label),
                        field_value,
                    )
                    .unwrap();

                    for line in value_lines {
                        if line.is_empty() {
                            content.push('\n');
                        } else {
                            writeln!(
                                content,
                                "{}{}",
                                " ".repeat(field_width + 1),
                                style(line).yellow()
                            )
                            .unwrap();
                        }
                    }
                }
                OutputEntry::Heading { text, indent } => {
                    writeln!(content, "{}{}", " ".repeat(indent), style(text).bold()).unwrap();
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

#[cfg(test)]
mod tests {
    use super::OutputBuilder;

    fn disable_colors() {
        console::set_colors_enabled(false);
    }

    #[test]
    fn indented_is_relative_to_current_indent() {
        disable_colors();
        let output = OutputBuilder::new()
            .indented(2, |b| b.indented(2, |b| b.field("ab", "1")))
            .build();

        assert_eq!(output, "    ab: 1");
    }

    #[test]
    fn heading_does_not_widen_label_column() {
        disable_colors();
        let output = OutputBuilder::new()
            .heading("A very long heading indeed")
            .field("a", "1")
            .build();

        assert_eq!(output, "A very long heading indeed\na: 1");
    }

    #[test]
    fn heading_uses_current_indent_in_nested_indented() {
        disable_colors();
        let output = OutputBuilder::new()
            .heading("h0")
            .indented(2, |b| {
                b.heading("h2")
                    .indented(3, |b| b.heading("h5").field("f5", "1"))
                    .heading("h2b")
                    .field("f2", "2")
            })
            .heading("h0b")
            .field("f0", "3")
            .build();

        assert_eq!(
            output,
            "h0\n  h2\n     h5\n     f5: 1\n  h2b\n  f2:    2\nh0b\nf0:      3"
        );
    }
}
