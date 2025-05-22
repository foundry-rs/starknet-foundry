use console::Style;
use serde::Serialize;

use crate::{Message, formats::NumbersFormat};

#[derive(Serialize)]
pub struct TypedMessage<'a> {
    tag: &'a str,
    message: &'a str,
    #[serde(skip)]
    type_style: Option<&'a str>,
}

impl<'a> TypedMessage<'a> {
    #[must_use]
    pub fn styled(tag: &'a str, message: &'a str, type_style: &'a str) -> Self {
        Self {
            tag,
            message,
            type_style: Some(type_style),
        }
    }
}

impl Message for TypedMessage<'_> {
    fn text(&self, numbers_format: NumbersFormat) -> String {
        let _ = numbers_format;
        format!(
            "{}: {}",
            self.type_style
                .map(Style::from_dotted_str)
                .unwrap_or_default()
                .apply_to(self.tag.to_string()),
            self.message
        )
    }
}
