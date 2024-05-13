use cairo_lang_macro::{Diagnostic, Diagnostics};

pub fn branch(
    left: impl Fn() -> Result<String, Diagnostic>,
    right: impl Fn() -> Result<String, Diagnostic>,
) -> Result<String, Diagnostics> {
    left().or_else(|error| right().map_err(|next_error| vec![error, next_error].into()))
}
