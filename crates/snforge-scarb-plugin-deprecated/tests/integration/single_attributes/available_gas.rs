use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN};
use cairo_lang_macro::{Diagnostic, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin_deprecated::attributes::available_gas::available_gas;

#[test]
fn works_with_empty() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("()".into());

    let result = available_gas(args, item);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std_deprecated::_internals::is_config_run() {
                    let mut data = array![];
                    snforge_std_deprecated::_internals::config_types::AvailableGasConfig::MaxResourceBounds(
                        snforge_std_deprecated::_internals::config_types::AvailableResourceBoundsConfig {
                        l1_gas: 0xffffffffffffffff,
                        l1_data_gas: 0xffffffffffffffff,
                        l2_gas: 0xffffffffffffffff
                        }
                    )
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
        &[Diagnostic::error(formatdoc!(
            "
                All options failed
                - variant: #[available_gas] <l2_gas> should be number literal
                - variant: #[available_gas] can be used with unnamed arguments only
                Resolve at least one of them
            "
        ))],
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
                if snforge_std_deprecated::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std_deprecated::_internals::config_types::AvailableGasConfig::MaxResourceBounds(
                        snforge_std_deprecated::_internals::config_types::AvailableResourceBoundsConfig {
                        l1_gas: 0x7b,
                        l1_data_gas: 0xffffffffffffffff,
                        l2_gas: 0xffffffffffffffff
                        }
                    )
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
                if snforge_std_deprecated::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std_deprecated::_internals::config_types::AvailableGasConfig::MaxResourceBounds(
                        snforge_std_deprecated::_internals::config_types::AvailableResourceBoundsConfig {
                        l1_gas: 0x1,
                        l1_data_gas: 0x2,
                        l2_gas: 0x3
                        }
                    )
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
        &[Diagnostic::error(formatdoc!(
            "
                All options failed
                - variant: <l2_gas> argument was specified 2 times, expected to be used only once
                - variant: #[available_gas] can be used with unnamed arguments only
                Resolve at least one of them
            "
        ))],
    );
}

#[test]
fn works_with_unnamed_number() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(3)".into());

    let result = available_gas(args, item);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std_deprecated::_internals::is_config_run() {
                    let mut data = array![];
                    snforge_std_deprecated::_internals::config_types::AvailableGasConfig::MaxGas(0x3)
                    .serialize(ref data);
                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());
                    return;
                }
            }
        ",
    );
}

// previously if some bonkers number was put into available_gas attribute, test always passed
// this was because u64 overflow, so now we test with u64::MAX + 1 to make sure it does not happen
#[test]
fn handles_number_overflow_unnamed() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(18446744073709551616)".into());

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "
                All options failed
                - variant: #[available_gas] can be used with named arguments only
                - variant: #[available_gas] max_gas it too large (max permissible value is 18446744073709551615)
                Resolve at least one of them
            "
        ))],
    );
}

#[test]
fn handles_number_overflow_l1() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(l1_gas: 18446744073709551616)".into());

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "
                All options failed
                - variant: #[available_gas] l1_gas it too large (max permissible value is 18446744073709551615)
                - variant: #[available_gas] can be used with unnamed arguments only
                Resolve at least one of them
            "
        ))],
    );
}

#[test]
fn handles_number_overflow_l1_data() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(l1_data_gas: 18446744073709551616)".into());

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "
                All options failed
                - variant: #[available_gas] l1_data_gas it too large (max permissible value is 18446744073709551615)
                - variant: #[available_gas] can be used with unnamed arguments only
                Resolve at least one of them
            "
        ))],
    );
}

#[test]
fn handles_number_overflow_l2() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(l2_gas: 18446744073709551616)".into());

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "
                All options failed
                - variant: #[available_gas] l2_gas it too large (max permissible value is 18446744073709551615)
                - variant: #[available_gas] can be used with unnamed arguments only
                Resolve at least one of them
            "
        ))],
    );
}

#[test]
fn max_permissible_value() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(l2_gas: 18446744073709551615)".into());

    let result = available_gas(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std_deprecated::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std_deprecated::_internals::config_types::AvailableGasConfig::MaxResourceBounds(
                        snforge_std_deprecated::_internals::config_types::AvailableResourceBoundsConfig {
                        l1_gas: 0xffffffffffffffff,
                        l1_data_gas: 0xffffffffffffffff,
                        l2_gas: 0xffffffffffffffff
                        }
                    )
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());

                    return;
                }
            }
        ",
    );
}
