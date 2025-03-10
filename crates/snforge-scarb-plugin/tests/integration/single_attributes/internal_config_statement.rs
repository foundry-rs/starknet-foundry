use crate::utils::{assert_diagnostics, assert_output};
use cairo_lang_macro::{quote, Diagnostic, TokenStream};
use snforge_scarb_plugin::attributes::internal_config_statement::internal_config_statement;

#[test]
fn fails_with_non_empty_args() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((123));

    let result = internal_config_statement(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[__internal_config_statement] does not accept any arguments",
        )],
    );
}
#[test]
fn appends_config_statement() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = internal_config_statement(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    return;
                }
            }
        ",
    );
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
