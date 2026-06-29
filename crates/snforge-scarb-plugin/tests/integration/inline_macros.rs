use cairo_lang_macro::TokenStream;
use snforge_scarb_plugin::create_single_token;

mod declare;

#[test]
fn declare_from_file_accepts_sierra_file_path() {
    let args = macro_args("\"tests/data/minimal.contract_class.json\"");

    let result = declare_from_file(&args);

    assert!(result.diagnostics.is_empty());
    insta::assert_snapshot!(result.token_stream.to_string());
}

#[test]
fn declare_from_file_rejects_non_string_literal() {
    let args = macro_args("tests::data");

    let result = declare_from_file(&args);

    assert_declare_from_file_diagnostic(
        &result,
        "`declare_from_file!` expects a string literal path to a Sierra contract class JSON file",
    );
}

#[test]
fn declare_from_file_rejects_missing_sierra_file() {
    let args = macro_args("\"tests/data/missing.contract_class.json\"");

    let result = declare_from_file(&args);

    assert_declare_from_file_diagnostic(
        &result,
        "Failed to read Sierra file at tests/data/missing.contract_class.json:",
    );
}

#[test]
fn declare_from_file_rejects_invalid_json_file() {
    let args = macro_args("\"tests/integration/inline_macros.rs\"");

    let result = declare_from_file(&args);

    assert_declare_from_file_diagnostic(
        &result,
        "Failed to parse Sierra contract class JSON at tests/integration/inline_macros.rs:",
    );
}

#[test]
fn declare_from_file_rejects_non_sierra_contract_class_json() {
    let args = macro_args("\"tests/data/invalid_contract_class.json\"");

    let result = declare_from_file(&args);

    assert_declare_from_file_diagnostic(
        &result,
        "File tests/data/invalid_contract_class.json is not a valid Sierra contract class JSON",
    );
}

fn macro_args(path: &str) -> TokenStream {
    TokenStream::new(vec![create_single_token(format!("({path})"))])
}
