use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN};
use cairo_lang_macro::{Diagnostic, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::available_gas::available_gas;

#[test]
fn fails_with_empty() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("()".into());

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
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
        &[Diagnostic::error(
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
        &[Diagnostic::error(
            "#[available_gas] <0> should be number literal",
        )],
    );
}

#[test]
fn fails_with_named() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(r#"(abc: "123")"#.into());

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[available_gas] can be used with unnamed attributes only",
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
                if snforge_std::_cheatcode::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::AvailableGasConfig {
                        gas: 0x7b
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());

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
        &[Diagnostic::error(
            "#[available_gas] can only be used once per item",
        )],
    );
}
