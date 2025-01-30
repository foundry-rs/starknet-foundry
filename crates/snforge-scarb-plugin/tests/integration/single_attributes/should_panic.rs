use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN};
use cairo_lang_macro::{Diagnostic, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::should_panic::should_panic;
use snforge_std::_internals::_is_config_run;
use snforge_std::_internals::ShouldPanicConfig;
use snforge_std::_internals::Expected;

#[test]
fn work_with_empty() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(String::new());

    let result = should_panic(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
            fn empty_fn() {
                if _is_config_run() {
                    let mut data = array![];

                    ShouldPanicConfig {
                        expected: snforge_std::_internals::_config_types::Expected::Any
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
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(r#"(expected: "panic data")"#.into());

    let result = should_panic(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    ShouldPanicConfig {
                        expected: snforge_std::_internals::_config_types::Expected::ByteArray("panic data")
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
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(r#"(expected: "can\"t \0 null byte")"#.into());

    let result = should_panic(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::internals::_config_types::ShouldPanicConfig {
                        expected: snforge_std::_internals::_config_types::Expected::ByteArray("can\"t \0 null byte")
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
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(r"(expected: 'panic data')".into());

    let result = should_panic(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::_config_types::ShouldPanicConfig {
                        expected: snforge_std::_internals::_config_types::Expected::ShortString('panic data')
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
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(r"(expected: 'can\'t')".into());

    let result = should_panic(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    ShouldPanicConfig {
                        expected: snforge_std::_internals::_config_types::Expected::ShortString('can\'t')
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
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(r"(expected: ('panic data', ' or not'))".into());

    let result = should_panic(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::_config_types::ShouldPanicConfig {
                        expected: snforge_std::_internals::_config_types::Expected::Array(array!['panic data',' or not',])
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
    let item = TokenStream::new(formatdoc!(
        "
            #[should_panic]
            {EMPTY_FN}
        "
    ));
    let args = TokenStream::new(String::new());

    let result = should_panic(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[should_panic] can only be used once per item",
        )],
    );
}
