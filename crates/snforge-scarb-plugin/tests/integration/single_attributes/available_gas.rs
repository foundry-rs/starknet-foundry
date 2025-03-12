use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN};
use cairo_lang_macro::{Diagnostic, TokenStream};
use snforge_scarb_plugin::attributes::available_gas::available_gas;

#[test]
fn works_with_empty() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("()".into());

    let result = available_gas(args, item);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];
                    snforge_std::_config_types::AvailableGasConfig {
                        l1_gas: 0xffffffffffffffff,
                        l1_data_gas: 0xffffffffffffffff,
                        l2_gas: 0xffffffffffffffff
                    }
                    .serialize(ref data);
                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());
                    return;
                }
            }
        ",
    );

    assert_diagnostics(
        &result,
        &[
            Diagnostic::warn("#[available_gas] used with empty argument list. Either remove () or specify some arguments"),
        ],
    );
}

#[test]
fn fails_with_non_number_literal() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(r#"(l2_gas: "123")"#.into());

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[available_gas] <l2_gas> should be number literal",
        )],
    );
}

#[test]
fn work_with_number_some_set() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(l1_gas: 123)".into());

    let result = available_gas(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::AvailableGasConfig {
                        l1_gas: 0x7b,
                        l1_data_gas: 0xffffffffffffffff,
                        l2_gas: 0xffffffffffffffff
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
fn work_with_number_all_set() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(l1_gas: 1, l1_data_gas: 2, l2_gas: 3)".into());

    let result = available_gas(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::AvailableGasConfig {
                        l1_gas: 0x1,
                        l1_data_gas: 0x2,
                        l2_gas: 0x3
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
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(l2_gas: 1, l2_gas: 3)".into());

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "<l2_gas> argument was specified 2 times, expected to be used only once",
        )],
    );
}

#[test]
fn does_not_work_with_unnamed_number() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(3)".into());

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[available_gas] can be used with named arguments only",
        )],
    );
}
