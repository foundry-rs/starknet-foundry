use crate::utils::{assert_diagnostics, assert_output};
use cairo_lang_macro::{quote, Diagnostic, TokenStream};
use snforge_scarb_plugin::attributes::test::test;

#[test]
fn appends_internal_config_and_executable() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = test(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[__internal_config_statement]
            #[snforge_internal_test_executable]
            fn empty_fn(){}
        ",
    );
}

#[test]
fn fails_with_non_empty_args() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((123));

    let result = test(args, item);

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
            "#[test] function with parameters must have #[fuzzer] attribute",
        )],
    );
}
