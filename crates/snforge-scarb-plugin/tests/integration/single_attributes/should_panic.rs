use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN};
use cairo_lang_macro::{Severity, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::should_panic::should_panic;

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
                if starknet::testing::cheatcode::<'is_config_mode'>() {
                    let mut data = array![];

                    snforge_std::_config_types::ShouldPanicConfig {
                        expected: snforge_std::_config_types::Expected::Any
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_should_panic'>(data);
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
                if starknet::testing::cheatcode::<'is_config_mode'>() {
                    let mut data = array![];

                    snforge_std::_config_types::ShouldPanicConfig {
                        expected: snforge_std::_config_types::Expected::ByteArray("panic data")
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_should_panic'>(data);
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
                if starknet::testing::cheatcode::<'is_config_mode'>() {
                    let mut data = array![];

                    snforge_std::_config_types::ShouldPanicConfig {
                        expected: snforge_std::_config_types::Expected::ShortString('panic data')
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_should_panic'>(data);
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
                if starknet::testing::cheatcode::<'is_config_mode'>() {
                    let mut data = array![];

                    snforge_std::_config_types::ShouldPanicConfig {
                        expected: snforge_std::_config_types::Expected::Array(array!['panic data',' or not',])
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_should_panic'>(data);
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
        &[(
            Severity::Error,
            "#[should_panic] can only be used once per item",
        )],
    );
}
