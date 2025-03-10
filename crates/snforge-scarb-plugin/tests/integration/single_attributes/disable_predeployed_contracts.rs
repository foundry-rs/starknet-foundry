use crate::utils::{assert_diagnostics, assert_output, empty_function};
use cairo_lang_macro::{quote, Diagnostic, TokenStream};
use snforge_scarb_plugin::attributes::disable_predeployed_contracts::disable_predeployed_contracts;

#[test]
fn fails_with_args() {
    let args = quote!((123));

    let result = disable_predeployed_contracts(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[disable_predeployed_contracts] does not accept any arguments",
        )],
    );
}

#[test]
fn works_without_args() {
    let args = TokenStream::empty();

    let result = disable_predeployed_contracts(args, empty_function());

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::PredeployedContractsConfig {
                        is_disabled: true
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_disable_contracts'>(data.span());

                    return;
                }
            }
        ",
    );
}

#[test]
fn is_used_once() {
    let item = quote! {
        #[disable_predeployed_contracts]
        fn empty_fn() {}
    };
    let args = TokenStream::empty();

    let result = disable_predeployed_contracts(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[disable_predeployed_contracts] can only be used once per item",
        )],
    );
}
