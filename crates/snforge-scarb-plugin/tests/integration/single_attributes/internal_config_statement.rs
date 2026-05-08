use crate::utils::{assert_diagnostics, empty_function, format_output};
use cairo_lang_macro::{Diagnostic, TokenStream, quote};
use snforge_scarb_plugin::attributes::internal_config_statement::internal_config_statement;

#[test]
fn fails_with_non_empty_args() {
    let args = quote!((123));

    let result = internal_config_statement(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[__internal_config_statement] does not accept any arguments",
        )],
    );
}

#[test]
fn appends_config_statement() {
    let args = TokenStream::empty();

    let result = internal_config_statement(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn is_used_once() {
    let item = quote!(
        #[__internal_config_statement]
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = internal_config_statement(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[__internal_config_statement] can only be used once per item",
        )],
    );
}
