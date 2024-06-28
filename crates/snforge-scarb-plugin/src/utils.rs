use cairo_lang_macro::{Diagnostic, Severity};
use indoc::formatdoc;

fn higher_severity(a: Severity, b: Severity) -> Severity {
    match (a, b) {
        (Severity::Warning, Severity::Warning) => Severity::Warning,
        _ => Severity::Error,
    }
}

pub fn branch(
    left: Result<String, Diagnostic>,
    right: impl Fn() -> Result<String, Diagnostic>,
) -> Result<String, Diagnostic> {
    left.or_else(|error| {
        right().map_err(|next_error| Diagnostic {
            severity: higher_severity(error.severity, next_error.severity),
            message: formatdoc!(
                "
                    Both options failed
                    First variant: {}
                    Second variant: {}
                    Resolve at least one of them
                ",
                error.message,
                next_error.message
            ),
        })
    })
}
