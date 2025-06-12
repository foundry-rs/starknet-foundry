use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN};
use cairo_lang_macro::{Diagnostic, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin_compatibility::attributes::ignore::ignore;

#[test]
fn fails_with_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(123)".into());

    let result = ignore(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error("#[ignore] does not accept any arguments")],
    );
}

#[test]
fn works_without_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(String::new());

    let result = ignore(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std_compatibility::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std_compatibility::_internals::config_types::IgnoreConfig {
                        is_ignored: true
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_ignore'>(data.span());

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
            #[ignore]
            {EMPTY_FN}
        "
    ));
    let args = TokenStream::new(String::new());

    let result = ignore(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[ignore] can only be used once per item",
        )],
    );
}
