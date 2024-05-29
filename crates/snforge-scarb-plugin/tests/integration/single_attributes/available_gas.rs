use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN};
use cairo_lang_macro::{Severity, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::available_gas::available_gas;

#[test]
fn fails_with_empty() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("()".into());

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[(
            Severity::Error,
            "#[available_gas] expected 1 arguments, got: 0",
        )],
    );
}

#[test]
fn fails_with_more_than_one() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(123,123,123)".into());

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[(
            Severity::Error,
            "#[available_gas] expected 1 arguments, got: 3",
        )],
    );
}

#[test]
fn fails_with_non_number_literal() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(r#"("123")"#.into());

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[(
            Severity::Error,
            "#[available_gas] <0> should be number literal",
        )],
    );
}

#[test]
fn work_with_number() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(123)".into());

    let result = available_gas(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if *starknet::testing::cheatcode::<'is_config_mode'>(array![].span()).at(0) == 1 {
                    let mut data = array![];

                    snforge_std::_config_types::AvailableGasConfig {
                        gas: 0x7b
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data);

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
            #[available_gas]
            {EMPTY_FN}
        "
    ));
    let args = TokenStream::new("(123)".into());

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[(
            Severity::Error,
            "#[available_gas] can only be used once per item",
        )],
    );
}
