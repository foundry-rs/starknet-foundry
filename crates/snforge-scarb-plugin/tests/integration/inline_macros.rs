use cairo_lang_macro::{Severity, quote};
use snforge_scarb_plugin::inline_macros::declare::declare;

#[test]
fn declare_accepts_contract_name() {
    let args = quote!(HelloStarknet);

    let result = declare(args);

    assert!(result.diagnostics.is_empty());
    insta::assert_snapshot!(result.token_stream.to_string());
}

#[test]
fn declare_accepts_full_module_path() {
    let args = quote!(my_package::hello_starknet::HelloStarknet);

    let result = declare(args);

    assert!(result.diagnostics.is_empty());
    insta::assert_snapshot!(result.token_stream.to_string());
}

#[test]
fn declare_accepts_partial_module_path() {
    let args = quote!(alias::HelloStarknet);

    let result = declare(args);

    assert!(result.diagnostics.is_empty());
    insta::assert_snapshot!(result.token_stream.to_string());
}

#[test]
fn declare_rejects_non_path_argument() {
    let args = quote!("HelloStarknet");

    let result = declare(args);

    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].severity(), Severity::Error);
    assert_eq!(
        result.diagnostics[0].message(),
        "`declare!` expects a contract module path like `HelloStarknet` or `my_package::my_module::MyContract`"
    );
}
