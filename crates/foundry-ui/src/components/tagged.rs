use console::Style;
use serde::Serialize;

use crate::Message;

/// Generic textual message with `tag` prefix.
///
/// The tag prefix can be stylized in text mode.
/// e.g. "[WARNING]: An example warning message"
#[derive(Serialize)]
pub struct TaggedMessage<'a> {
    tag: &'a str,
    text: &'a str,

    /// Field which dictates the style of the tag as a string that `console::Style` can interpret.
    #[serde(skip)]
    tag_style: Option<&'a str>,
}

impl<'a> TaggedMessage<'a> {
    #[must_use]
    pub fn styled(tag: &'a str, text: &'a str, tag_style: &'a str) -> Self {
        Self {
            tag,
            text,
            tag_style: Some(tag_style),
        }
    }

    #[must_use]
    pub fn raw(tag: &'a str, text: &'a str) -> Self {
        Self {
            tag,
            text,
            tag_style: None,
        }
    }
}

impl Message for TaggedMessage<'_> {
    fn text(&self) -> String {
        format!(
            "[{}] {}",
            self.tag_style
                .map(Style::from_dotted_str)
                .unwrap_or_default()
                .apply_to(self.tag.to_string()),
            self.text
        )
    }
}
