use crate::utils::{assert_diagnostics, empty_function, format_output};
use cairo_lang_macro::{Diagnostic, TokenStream, quote};
use snforge_scarb_plugin::attributes::disable_predeployed_contracts::disable_predeployed_contracts;

#[test]
fn fails_with_args() {
    let args = quote!((123));

    let result = disable_predeployed_contracts(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[disable_predeployed_contracts] does not accept any arguments",
        )],
    );
}

#[test]
fn works_without_args() {
    let args = TokenStream::empty();

    let result = disable_predeployed_contracts(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn is_used_once() {
    let item = quote! {
        #[disable_predeployed_contracts]
        fn empty_fn() {}
    };
    let args = TokenStream::empty();

    let result = disable_predeployed_contracts(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[disable_predeployed_contracts] can only be used once per item",
        )],
    );
}
