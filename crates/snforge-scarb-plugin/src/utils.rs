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
        right().map_err(|next_error| {
            let next_message = if next_error.message.contains("All options failed") {
                let mut lines: Vec<&str> = next_error.message.lines().collect();
                lines = lines[1..lines.len() - 1].to_vec();
                let mut next_message = lines.join("\n");
                if let Some(pos) = next_message.find("- variant: ") {
                    next_message.replace_range(pos..pos + 11, "");
                }
                next_message
            } else {
                next_error.message.clone()
            };

            Diagnostic {
                severity: higher_severity(error.severity, next_error.severity),
                message: formatdoc!(
                    "
                        All options failed
                        - variant: {}
                        - variant: {}
                        Resolve at least one of them
                    ",
                    error.message,
                    next_message
                ),
            }
        })
    })
}
