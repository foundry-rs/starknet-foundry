use console::Style;
use serde::Serialize;

use crate::Message;

/// Generic textual message with `ty` prefix.
///
/// The type prefix can be stylized in text mode.
/// e.g. "Tests: 1 passed, 1 failed"
#[derive(Serialize)]
pub struct LabeledMessage<'a> {
    label: &'a str,
    text: &'a str,
    #[serde(skip)]
    label_style: Option<&'a str>,
}

impl<'a> LabeledMessage<'a> {
    #[must_use]
    pub fn styled(label: &'a str, text: &'a str, label_style: &'a str) -> Self {
        Self {
            label,
            text,
            label_style: Some(label_style),
        }
    }

    #[must_use]
    pub fn raw(label: &'a str, text: &'a str) -> Self {
        Self {
            label,
            text,
            label_style: None,
        }
    }
}

impl Message for LabeledMessage<'_> {
    fn text(&self) -> String {
        format!(
            "{}: {}",
            self.label_style
                .map(Style::from_dotted_str)
                .unwrap_or_default()
                .apply_to(self.label.to_string()),
            self.text
        )
    }
}
