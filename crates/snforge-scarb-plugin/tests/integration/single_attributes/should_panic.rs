use crate::utils::{assert_diagnostics, empty_function, format_output};
use cairo_lang_macro::{Diagnostic, TextSpan, Token, TokenStream, TokenTree, quote};
use snforge_scarb_plugin::attributes::should_panic::should_panic;

#[test]
fn work_with_empty() {
    let args = TokenStream::empty();

    let result = should_panic(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn work_with_expected_string() {
    let args = quote!((expected: "panic data"));

    let result = should_panic(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn work_with_expected_string_escaped() {
    let args = quote!((expected: "can\"t \0 null byte"));

    let result = should_panic(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn work_with_expected_short_string() {
    let args = TokenStream::new(vec![TokenTree::Ident(Token::new(
        "(expected: 'panic data')",
        TextSpan::call_site(),
    ))]);

    let result = should_panic(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn work_with_expected_short_string_escaped() {
    let args = TokenStream::new(vec![TokenTree::Ident(Token::new(
        r"(expected: 'can\'t')",
        TextSpan::call_site(),
    ))]);

    let result = should_panic(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn work_with_expected_tuple() {
    let args = TokenStream::new(vec![TokenTree::Ident(Token::new(
        r"(expected: ('panic data', ' or not'))",
        TextSpan::call_site(),
    ))]);

    let result = should_panic(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn is_used_once() {
    let item = quote!(
        #[should_panic]
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = should_panic(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[should_panic] can only be used once per item",
        )],
    );
}

#[test]
fn fails_with_unexpected_args() {
    let args = quote!((expected: "panic", tomato: 123));

    let result = should_panic(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[should_panic] unexpected argument(s): <tomato>",
        )],
    );
}

#[test]
fn fails_with_unnamed_arg() {
    let args = quote!(("uwu"));

    let result = should_panic(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[should_panic] can be used with named arguments only [possible values: expected]",
        )],
    );
}
