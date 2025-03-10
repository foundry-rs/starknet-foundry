use crate::utils::{assert_diagnostics, assert_output, empty_function};
use cairo_lang_macro::{quote, Diagnostic, TextSpan, Token, TokenStream, TokenTree};
use snforge_scarb_plugin::attributes::should_panic::should_panic;

#[test]
fn work_with_empty() {
    let args = TokenStream::empty();

    let result = should_panic(args, empty_function());

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::ShouldPanicConfig {
                        expected: snforge_std::_internals::config_types::Expected::Any
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_should_panic'>(data.span());
                    return;
                }
            }
        ",
    );
}

#[test]
fn work_with_expected_string() {
    let args = quote!((expected: "panic data"));

    let result = should_panic(args, empty_function());

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::ShouldPanicConfig {
                        expected: snforge_std::_internals::config_types::Expected::ByteArray("panic data")
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_should_panic'>(data.span());
                    return;
                }
            }
        "#,
    );
}

#[test]
fn work_with_expected_string_escaped() {
    let args = quote!((expected: "can\"t \0 null byte"));

    let result = should_panic(args, empty_function());

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::ShouldPanicConfig {
                        expected: snforge_std::_internals::config_types::Expected::ByteArray("can\"t \0 null byte")
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_should_panic'>(data.span());
                    return;
                }
            }
        "#,
    );
}

#[test]
fn work_with_expected_short_string() {
    let args = TokenStream::new(vec![TokenTree::Ident(Token::new(
        "(expected: 'panic data')",
        TextSpan::call_site(),
    ))]);

    let result = should_panic(args, empty_function());

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::ShouldPanicConfig {
                        expected: snforge_std::_internals::config_types::Expected::ShortString('panic data')
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_should_panic'>(data.span());
                    return;
                }
            }
        ",
    );
}

#[test]
fn work_with_expected_short_string_escaped() {
    let args = TokenStream::new(vec![TokenTree::Ident(Token::new(
        r"(expected: 'can\'t')",
        TextSpan::call_site(),
    ))]);

    let result = should_panic(args, empty_function());

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::ShouldPanicConfig {
                        expected: snforge_std::_internals::config_types::Expected::ShortString('can\'t')
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_should_panic'>(data.span());
                    return;
                }
            }
        ",
    );
}

#[test]
fn work_with_expected_tuple() {
    let args = TokenStream::new(vec![TokenTree::Ident(Token::new(
        r"(expected: ('panic data', ' or not'))",
        TextSpan::call_site(),
    ))]);

    let result = should_panic(args, empty_function());

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::ShouldPanicConfig {
                        expected: snforge_std::_internals::config_types::Expected::Array(array!['panic data',' or not',])
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_should_panic'>(data.span());
                    return;
                }
            }
        ",
    );
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
