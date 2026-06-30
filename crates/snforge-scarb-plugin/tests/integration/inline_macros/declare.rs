use super::macro_args;
use cairo_lang_macro::Severity;
use snforge_scarb_plugin::inline_macros::declare::declare;

#[test]
fn accepts_contract_name() {
    let args = macro_args("HelloStarknet");

    let result = declare(&args);

    assert!(result.diagnostics.is_empty());
    insta::assert_snapshot!(result.token_stream.to_string());
}

#[test]
fn accepts_full_module_path() {
    let args = macro_args("my_package::hello_starknet::HelloStarknet");

    let result = declare(&args);

    assert!(result.diagnostics.is_empty());
    insta::assert_snapshot!(result.token_stream.to_string());
}

#[test]
fn accepts_partial_module_path() {
    let args = macro_args("hello_starknet::HelloStarknet");

    let result = declare(&args);

    assert!(result.diagnostics.is_empty());
    insta::assert_snapshot!(result.token_stream.to_string());
}

#[test]
fn rejects_non_path_argument() {
    let args = macro_args("\"HelloStarknet\"");

    let result = declare(&args);

    assert_declare_path_diagnostic(&result);
}

#[test]
fn rejects_whitespace_between_identifiers() {
    let args = macro_args("Hello Starknet");

    let result = declare(&args);

    assert_declare_path_diagnostic(&result);
}

#[test]
fn rejects_wrapped_path() {
    let args = macro_args("(HelloStarknet)");

    let result = declare(&args);

    assert_declare_path_diagnostic(&result);
}

fn assert_declare_path_diagnostic(result: &cairo_lang_macro::ProcMacroResult) {
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity(), Severity::Error);
    assert_eq!(
        result.diagnostics[0].message(),
        "`declare!` expects either a contract name (e.g. `MyContract`), an absolute module tree path (e.g. `my_package::module::MyContract`) or a partial module tree path (e.g. `module::MyContract`)",
    );
}
