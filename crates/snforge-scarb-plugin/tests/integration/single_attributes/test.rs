use crate::utils::{assert_diagnostics, empty_function, format_output};
use cairo_lang_macro::{Diagnostic, TokenStream, quote};
use snforge_scarb_plugin::attributes::test::test;

#[test]
fn appends_internal_config_and_executable() {
    let args = TokenStream::empty();

    let result = test(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn fails_with_non_empty_args() {
    let args = quote!((123));

    let result = test(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error("#[test] does not accept any arguments")],
    );
}

#[test]
fn is_used_once() {
    let item = quote!(
        #[test]
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = test(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error("#[test] can only be used once per item")],
    );
}

#[test]
fn fails_with_params() {
    let item = quote!(
        fn empty_fn(f: felt252) {}
    );
    let args = TokenStream::empty();

    let result = test(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[test] function with parameters must have #[fuzzer] or #[test_case] attribute",
        )],
    );
}
