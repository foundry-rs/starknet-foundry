use cairo_lang_macro::{Diagnostic, Severity};
use indoc::formatdoc;

pub fn higher_severity(a: Severity, b: Severity) -> Severity {
    match (a, b) {
        (Severity::Warning, Severity::Warning) => Severity::Warning,
        _ => Severity::Error,
    }
}
pub fn format_error_message(variants: &[Diagnostic]) -> String {
    let formatted_variants: Vec<String> = variants
        .iter()
        .map(|variant| format!("- variant: {}", variant.message))
        .collect();

    formatdoc! {"
        All options failed
        {}
        Resolve at least one of them
    ", formatted_variants.join("\n")}
}

#[macro_export]
macro_rules! branch {
    ($($result:expr),+) => {{
        let mut messages = Vec::new();
        let mut result = None;

        $(
            if result.is_none() {match $result {
                Ok(val) => {
                    result = Some(val);
                },
                Err(err) => {
                    messages.push(err);
                },
            }}
        )+
        if let Some(result) = result {
            Ok(result)
        } else {
            Err(Diagnostic {
                message: crate::utils::format_error_message(&messages),
                severity: messages.into_iter().fold(Severity::Warning, |acc, x| crate::utils::higher_severity(acc, x.severity))
            })
        }
    }};
}
