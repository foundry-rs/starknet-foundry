use console::Style;
use serde::Serialize;

use crate::Message;

#[derive(Serialize)]
pub struct TypedMessage<'a> {
    ty: &'a str,
    text: &'a str,
    #[serde(skip)]
    type_style: Option<&'a str>,
}

impl<'a> TypedMessage<'a> {
    #[must_use]

    pub fn styled(ty: &'a str, text: &'a str, type_style: &'a str) -> Self {
        Self {
            ty,
            text,
            type_style: Some(type_style),
        }
    }
}

impl Message for TypedMessage<'_> {
    fn text(&self) -> String {
        format!(
            "{}: {}",
            self.type_style
                .map(Style::from_dotted_str)
                .unwrap_or_default()
                .apply_to(self.ty.to_string()),
            self.text
        )
    }
}
