use super::macro_args;
use cairo_lang_macro::Severity;
use snforge_scarb_plugin::inline_macros::declare_from_file::declare_from_file;

#[test]
fn accepts_sierra_file_path() {
    let args = macro_args("\"tests/data/minimal.contract_class.json\"");

    let result = declare_from_file(&args);

    assert!(result.diagnostics.is_empty());
    insta::assert_snapshot!(result.token_stream.to_string());
}

#[test]
fn accepts_sierra_file_path_with_trailing_comma() {
    let args = macro_args("\"tests/data/minimal.contract_class.json\",");

    let result = declare_from_file(&args);

    assert!(result.diagnostics.is_empty());
    assert_eq!(
        result.token_stream.to_string(),
        "snforge_std::declare_from_file(\"tests/data/minimal.contract_class.json\")"
    );
}

#[test]
fn rejects_non_string_literal() {
    let args = macro_args("tests::data");

    let result = declare_from_file(&args);

    assert_declare_from_file_diagnostic(
        &result,
        "`declare_from_file!` expects a string literal path to a Sierra contract class JSON file",
    );
}

#[test]
fn rejects_missing_sierra_file() {
    let args = macro_args("\"tests/data/missing.contract_class.json\"");

    let result = declare_from_file(&args);

    assert_declare_from_file_diagnostic(
        &result,
        "Failed to read Sierra file at tests/data/missing.contract_class.json:",
    );
}

#[test]
fn rejects_invalid_json_file() {
    let args = macro_args("\"tests/integration/inline_macros.rs\"");

    let result = declare_from_file(&args);

    assert_declare_from_file_diagnostic(
        &result,
        "Failed to parse Sierra contract class JSON at tests/integration/inline_macros.rs:",
    );
}

#[test]
fn rejects_non_sierra_contract_class_json() {
    let args = macro_args("\"tests/data/invalid_contract_class.json\"");

    let result = declare_from_file(&args);

    assert_declare_from_file_diagnostic(
        &result,
        "File tests/data/invalid_contract_class.json is not a valid Sierra contract class JSON",
    );
}

fn assert_declare_from_file_diagnostic(
    result: &cairo_lang_macro::ProcMacroResult,
    expected_message_prefix: &str,
) {
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity(), Severity::Error);
    assert!(
        result.diagnostics[0]
            .message()
            .starts_with(expected_message_prefix),
        "expected diagnostic to start with `{expected_message_prefix}`, got `{}`",
        result.diagnostics[0].message()
    );
}
