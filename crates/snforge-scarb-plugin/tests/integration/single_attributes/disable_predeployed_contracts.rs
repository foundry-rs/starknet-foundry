use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN};
use cairo_lang_macro::{Diagnostic, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::disable_predeployed_contracts::disable_predeployed_contracts;

#[test]
fn fails_with_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(123)".into());

    let result = disable_predeployed_contracts(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[disable_predeployed_contracts] does not accept any arguments",
        )],
    );
}

#[test]
fn works_without_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(String::new());

    let result = disable_predeployed_contracts(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::ContractsPredeploymentConfig {
                        is_disabled: true
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_contracts_predeployment'>(data.span());

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
            #[disable_predeployed_contracts]
            {EMPTY_FN}
        "
    ));
    let args = TokenStream::new(String::new());

    let result = disable_predeployed_contracts(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[disable_predeployed_contracts] can only be used once per item",
        )],
    );
}
