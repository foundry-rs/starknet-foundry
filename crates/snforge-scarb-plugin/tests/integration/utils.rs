use cairo_lang_macro::{ProcMacroResult, Severity};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

pub const EMPTY_FN: &str = "fn empty_fn(){}";

pub fn assert_diagnostics(result: &ProcMacroResult, expected: &[(Severity, &str)]) {
    let diagnostics: HashSet<_> = result
        .diagnostics
        .iter()
        .map(|d| {
            (
                match d.severity {
                    Severity::Error => 0,
                    Severity::Warning => 1,
                },
                d.message.as_str(),
            )
        })
        .collect();
    let expected: HashSet<_> = expected
        .iter()
        .map(|d| {
            (
                match d.0 {
                    Severity::Error => 0,
                    Severity::Warning => 1,
                },
                d.1,
            )
        })
        .collect();

    let remaining_diagnostics: Vec<_> = diagnostics
        .difference(&expected)
        .map(|diff| {
            format!(
                "Diagnostic where emitted and unexpected:\n {} => \"{}\"\n",
                match diff.0 {
                    0 => "Severity::Error",
                    _ => "Severity::Warning",
                },
                diff.1,
            )
        })
        .collect();

    let not_emitted_diagnostics: Vec<_> = expected
        .difference(&diagnostics)
        .map(|diff| {
            format!(
                "Diagnostic where expected but not emitted:\n {} => \"{}\"\n",
                match diff.0 {
                    0 => "Severity::Error",
                    _ => "Severity::Warning",
                },
                diff.1,
            )
        })
        .collect();

    assert!(
        remaining_diagnostics.is_empty() && not_emitted_diagnostics.is_empty(),
        "\n---------------------\n\n{}\n---------------------",
        remaining_diagnostics.join("\n") + "\n" + &not_emitted_diagnostics.join("\n")
    );
}

// generated code is terribly formatted so replace all whitespace sequences with single one
// this wont work if we emit string literals with whitespaces
// but we don't others than user provided ones and it's faster and easier than scarb fmt
pub fn assert_output(result: &ProcMacroResult, expected: &str) {
    lazy_static! {
        static ref WHITESPACES: Regex = Regex::new(r"\s+").unwrap();
    }
    assert_eq!(
        WHITESPACES
            .replace_all(&result.token_stream.to_string(), " ")
            .trim(),
        WHITESPACES.replace_all(expected, " ").trim(),
        "invalid code generated"
    );
}
