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
    middle: impl Fn() -> Result<String, Diagnostic>,
    right: impl Fn() -> Result<String, Diagnostic>,
) -> Result<String, Diagnostic> {
    left.or_else(|left_error| {
        middle().or_else(|middle_error| {
            right().map_err(|right_error| Diagnostic {
                severity: higher_severity(
                    higher_severity(left_error.severity, middle_error.severity),
                    right_error.severity,
                ),
                message: formatdoc!(
                    "
                        All options failed
                        First variant: {}
                        Second variant: {}
                        Third variant: {}
                        Resolve at least one of them
                    ",
                    left_error.message,
                    middle_error.message,
                    right_error.message
                ),
            })
        })
    })
}
