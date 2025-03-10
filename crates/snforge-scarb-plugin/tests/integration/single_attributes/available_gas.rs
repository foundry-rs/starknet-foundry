use crate::utils::{assert_diagnostics, assert_output};
use cairo_lang_macro::{quote, Diagnostic};
use snforge_scarb_plugin::attributes::available_gas::available_gas;

#[test]
fn fails_with_empty() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!(());

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[
            Diagnostic::warn("#[available_gas] used with empty argument list. Either remove () or specify some arguments"),
            Diagnostic::error(
            "#[available_gas] expected arguments: 1, got: 0",
        )],
    );
}

#[test]
fn fails_with_more_than_one() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((1, 2, 3));

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[available_gas] expected arguments: 1, got: 3",
        )],
    );
}

#[test]
fn fails_with_non_number_literal() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!(("123"));

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[available_gas] <0> should be number literal",
        )],
    );
}

#[test]
fn work_with_number() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((123));

    let result = available_gas(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];
                    snforge_std::_config_types::AvailableGasConfig {
                        gas: 0x7b
                    }
                    .serialize(ref data);
                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());
                    return;
                }
            }",
    );
}

#[test]
fn is_used_once() {
    let item = quote!(
        #[available_gas]
        fn empty_fn() {}
    );
    let args = quote!((123));

    let result = available_gas(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[available_gas] can only be used once per item",
        )],
    );
}
