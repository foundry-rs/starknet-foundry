use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN};
use cairo_lang_macro::{Diagnostic, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::disable_strk_predeployment::disable_strk_predeployment;

#[test]
fn fails_with_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(123)".into());

    let result = disable_strk_predeployment(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[disable_strk_predeployment] does not accept any arguments",
        )],
    );
}

#[test]
fn works_without_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(String::new());

    let result = disable_strk_predeployment(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::StrkPredeploymentConfig {
                        is_disabled: true
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_strk_predeployment'>(data.span());

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
            #[disable_strk_predeployment]
            {EMPTY_FN}
        "
    ));
    let args = TokenStream::new(String::new());

    let result = disable_strk_predeployment(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[disable_strk_predeployment] can only be used once per item",
        )],
    );
}
