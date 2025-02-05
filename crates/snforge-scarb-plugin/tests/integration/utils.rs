use cairo_lang_macro::{Diagnostic, ProcMacroResult};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

pub const EMPTY_FN: &str = "fn empty_fn(){}";
pub const FN_WITH_SINGLE_FELT252_PARAM: &str = "fn empty_fn(f: felt252){}";

pub fn assert_diagnostics(result: &ProcMacroResult, expected: &[Diagnostic]) {
    let diagnostics: HashSet<_> = result.diagnostics.iter().collect();
    let expected: HashSet<_> = expected.iter().collect();

    let remaining_diagnostics: Vec<_> = diagnostics
        .difference(&expected)
        .map(|diff| {
            format!(
                "Diagnostic where emitted and unexpected:\n {:?} => \"{}\"\n",
                diff.severity, diff.message,
            )
        })
        .collect();

    let not_emitted_diagnostics: Vec<_> = expected
        .difference(&diagnostics)
        .map(|diff| {
            format!(
                "Diagnostic where expected but not emitted:\n {:?} => \"{}\"\n",
                diff.severity, diff.message,
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
