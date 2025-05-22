use console::Style;
use serde::Serialize;

use crate::Message;

#[derive(Serialize)]
pub struct TaggedMessage<'a> {
    tag: &'a str,
    text: &'a str,
    #[serde(skip)]
    type_style: Option<&'a str>,
}

impl<'a> TaggedMessage<'a> {
    #[must_use]
    pub fn styled(tag: &'a str, text: &'a str, type_style: &'a str) -> Self {
        Self {
            tag,
            text,
            type_style: Some(type_style),
        }
    }
}

impl Message for TaggedMessage<'_> {
    fn text(&self) -> String {
        format!(
            "[{}]: {}",
            self.type_style
                .map(Style::from_dotted_str)
                .unwrap_or_default()
                .apply_to(self.tag.to_string()),
            self.text
        )
    }
}
