use cairo_lang_formatter::{CairoFormatter, FormatterConfig};
use cairo_lang_macro::{Diagnostic, ProcMacroResult};
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

// Before asserting output token, format both strings with `CairoFormatter` and normalize by removing newlines
pub fn assert_output(result: &ProcMacroResult, expected: &str) {
    let fmt = CairoFormatter::new(FormatterConfig::default());
    let format_and_normalize_code = |code: String| -> String {
        fmt.format_to_string(&code)
            .unwrap()
            .into_output_text()
            .replace('\n', "")
            .trim()
            .to_string()
    };

    assert_eq!(
        format_and_normalize_code(result.token_stream.to_string()),
        format_and_normalize_code(expected.to_string()),
        "Invalid code generated"
    );
}
