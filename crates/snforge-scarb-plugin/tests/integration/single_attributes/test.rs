use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN, FN_WITH_SINGLE_FELT252_PARAM};
use cairo_lang_macro::{Diagnostic, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::test::test;

#[test]
fn appends_internal_config_and_executable() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(String::new());

    let result = test(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn(){}
        ",
    );
}

#[test]
fn fails_with_non_empty_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(123)".into());

    let result = test(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error("#[test] does not accept any arguments")],
    );
}

#[test]
fn is_used_once() {
    let item = TokenStream::new(formatdoc!(
        "
            #[test]
            {EMPTY_FN}
        "
    ));
    let args = TokenStream::new(String::new());

    let result = test(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error("#[test] can only be used once per item")],
    );
}

#[test]
fn fails_with_params() {
    let item = TokenStream::new(FN_WITH_SINGLE_FELT252_PARAM.into());
    let args = TokenStream::new(String::new());

    let result = test(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[test] function with parameters must have #[fuzzer] attribute",
        )],
    );
}
