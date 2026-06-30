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
fn accepts_sierra_file_with_debug_info_annotations() {
    let args = macro_args("\"tests/data/minimal_with_annotations.contract_class.json\"");

    let result = declare_from_file(&args);

    assert!(result.diagnostics.is_empty());
    assert_eq!(
        result.token_stream.to_string(),
        "snforge_std::declare_from_file(\"tests/data/minimal_with_annotations.contract_class.json\")"
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
fn accepts_missing_sierra_file_path() {
    let args = macro_args("\"tests/data/missing.contract_class.json\"");

    let result = declare_from_file(&args);

    assert!(result.diagnostics.is_empty());
    assert_eq!(
        result.token_stream.to_string(),
        "snforge_std::declare_from_file(\"tests/data/missing.contract_class.json\")"
    );
}

#[test]
fn accepts_invalid_json_file_path() {
    let args = macro_args("\"tests/integration/inline_macros.rs\"");

    let result = declare_from_file(&args);

    assert!(result.diagnostics.is_empty());
    assert_eq!(
        result.token_stream.to_string(),
        "snforge_std::declare_from_file(\"tests/integration/inline_macros.rs\")"
    );
}

#[test]
fn accepts_non_sierra_contract_class_json_path() {
    let args = macro_args("\"tests/data/invalid_contract_class.json\"");

    let result = declare_from_file(&args);

    assert!(result.diagnostics.is_empty());
    assert_eq!(
        result.token_stream.to_string(),
        "snforge_std::declare_from_file(\"tests/data/invalid_contract_class.json\")"
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
