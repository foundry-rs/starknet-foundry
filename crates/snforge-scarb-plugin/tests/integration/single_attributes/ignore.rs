use crate::utils::{assert_diagnostics, empty_function, format_output};
use cairo_lang_macro::{Diagnostic, TokenStream, quote};
use snforge_scarb_plugin::attributes::ignore::ignore;

#[test]
fn fails_with_args() {
    let args = quote!((123));

    let result = ignore(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error("#[ignore] does not accept any arguments")],
    );
}

#[test]
fn works_without_args() {
    let args = TokenStream::empty();

    let result = ignore(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn is_used_once() {
    let item = quote!(
        #[ignore]
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = ignore(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[ignore] can only be used once per item",
        )],
    );
}
