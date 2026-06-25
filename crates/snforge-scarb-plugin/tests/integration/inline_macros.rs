use cairo_lang_macro::{quote, Severity};
use snforge_scarb_plugin::inline_macros::declare::declare;

#[test]
fn declare_accepts_contract_name() {
    let args = quote!(HelloStarknet);

    let result = declare(&args);

    assert!(result.diagnostics.is_empty());
    insta::assert_snapshot!(result.token_stream.to_string());
}

#[test]
fn declare_accepts_full_module_path() {
    let args = quote!(my_package::hello_starknet::HelloStarknet);

    let result = declare(&args);

    assert!(result.diagnostics.is_empty());
    insta::assert_snapshot!(result.token_stream.to_string());
}

#[test]
fn declare_accepts_partial_module_path() {
    let args = quote!(hello_starknet::HelloStarknet);

    let result = declare(&args);

    assert!(result.diagnostics.is_empty());
    insta::assert_snapshot!(result.token_stream.to_string());
}

#[test]
fn declare_rejects_non_path_argument() {
    let args = quote!("HelloStarknet");

    let result = declare(&args);

    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity(), Severity::Error);
    assert_eq!(
        result.diagnostics[0].message(),
        "`declare!` expects either a contract name (e.g. `MyContract`), an absolute module tree path (e.g. `my_package::module::MyContract`) or a partial module tree path (e.g. `module::MyContract`)",
    );
}
