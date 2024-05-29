use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN};
use cairo_lang_macro::{Severity, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::ignore::ignore;

#[test]
fn fails_with_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(123)".into());

    let result = ignore(args, item);

    assert_diagnostics(
        &result,
        &[(Severity::Error, "#[ignore] does not accept any arguments")],
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
                if *starknet::testing::cheatcode::<'is_config_mode'>(array![].span()).at(0) == 1 {
                    let mut data = array![];

                    snforge_std::_config_types::IgnoreConfig {
                        is_ignored: true
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_ignore'>(data);

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
        &[(Severity::Error, "#[ignore] can only be used once per item")],
    );
}
