use crate::utils::{assert_diagnostics, assert_output};
use cairo_lang_macro::{quote, Diagnostic, TokenStream};
use snforge_scarb_plugin::attributes::ignore::ignore;

#[test]
fn fails_with_args() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((123));

    let result = ignore(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error("#[ignore] does not accept any arguments")],
    );
}

#[test]
fn works_without_args() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = ignore(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::IgnoreConfig {
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
    let item = quote!(
        #[ignore]
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = ignore(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[ignore] can only be used once per item",
        )],
    );
}
