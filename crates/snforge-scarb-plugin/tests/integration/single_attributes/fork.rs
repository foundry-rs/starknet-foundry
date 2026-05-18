use crate::utils::{assert_diagnostics, empty_function, format_output};
use cairo_lang_macro::{Diagnostic, quote};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::fork::fork;

#[test]
fn fails_without_block() {
    let args = quote!((url: "invalid url"));

    let result = fork(args, empty_function());

    assert_diagnostics(
        &result,
        &[
           Diagnostic::error(formatdoc!(
                "
                    All options failed
                    - variant: exactly one of <block_hash> | <block_number> | <block_tag> should be specified, got 0
                    - variant: #[fork] expected arguments: 1, got: 0
                    - variant: #[fork] can be used with unnamed arguments only
                    Resolve at least one of them
                "
            ))
        ],
    );
}

#[test]
fn fails_without_url() {
    let args = quote!((block_number: 23));

    let result = fork(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "
                All options failed
                - variant: <url> argument is missing
                - variant: #[fork] expected arguments: 1, got: 0
                - variant: #[fork] can be used with unnamed arguments only
                Resolve at least one of them
            "
        ))],
    );
}

#[test]
fn fails_without_args() {
    let args = quote!(());

    let result = fork(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::warn("#[fork] used with empty argument list. Either remove () or specify some arguments"),
            Diagnostic::error(formatdoc!(
            "
                All options failed
                - variant: exactly one of <block_hash> | <block_number> | <block_tag> should be specified, got 0
                - variant: #[fork] expected arguments: 1, got: 0
                - variant: #[fork] expected arguments: 1, got: 0
                Resolve at least one of them
            "
        ))],
    );
}

#[test]
fn fails_with_invalid_url() {
    let args = quote!((url: "invalid url", block_number: 23));

    let result = fork(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "
                All options failed
                - variant: #[fork] <url> is not a valid url
                - variant: #[fork] expected arguments: 1, got: 0
                - variant: #[fork] can be used with unnamed arguments only
                Resolve at least one of them
            "
        ))],
    );
}

#[test]
fn accepts_string() {
    let args = quote!(("test"));

    let result = fork(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn fails_with_unexpected_args() {
    let args = quote!((url: "http://example.com", block_number: 23, tomato: 123, hello: "world"));

    let result = fork(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "
                All options failed
                - variant: #[fork] unexpected argument(s): <hello>, <tomato>
                - variant: #[fork] expected arguments: 1, got: 0
                - variant: #[fork] can be used with unnamed arguments only
                Resolve at least one of them
            "
        ))],
    );
}

#[test]
fn accepts_inline_config() {
    let args = quote!((url: "http://example.com", block_number: 23));

    let result = fork(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn overriding_config_name_first() {
    let args = quote!(("MAINNET", block_number: 23));

    let result = fork(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn overriding_config_name_second() {
    let args = quote!((block_number: 23, "MAINNET"));

    let result = fork(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn is_used_once() {
    let item = quote!(
        #[fork]
        fn empty_fn() {}
    );
    let args = quote!(("name"));

    let result = fork(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error("#[fork] can only be used once per item")],
    );
}
