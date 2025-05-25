use console::Style;
use serde::Serialize;

use crate::Message;

/// Generic textual message with applied style.
#[derive(Serialize)]
pub struct StyledMessage<'a> {
    text: &'a str,
    #[serde(skip)]
    text_style: &'a str,
}

impl<'a> StyledMessage<'a> {
    #[must_use]
    pub fn new(text: &'a str, text_style: &'a str) -> Self {
        Self { text, text_style }
    }
}

impl Message for StyledMessage<'_> {
    fn text(&self) -> String {
        let style = Style::from_dotted_str(self.text_style);
        style.apply_to(self.text.to_string()).to_string()
    }
}
