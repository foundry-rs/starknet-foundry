use crate::utils::{assert_diagnostics, empty_function, format_output};
use cairo_lang_macro::{Diagnostic, TokenStream, quote};
use snforge_scarb_plugin::attributes::extract_config::extract_config;

#[test]
fn fails_with_non_empty_args() {
    let args = quote!((123));

    let result = extract_config(args, empty_function());

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[__snforge_extract_config] does not accept any arguments",
        )],
    );
}

#[test]
fn extracts_config_function() {
    let args = TokenStream::empty();

    let result = extract_config(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::assert_snapshot!(format_output(&result));
}

#[test]
fn is_used_once() {
    let item = quote!(
        #[__snforge_extract_config]
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = extract_config(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[__snforge_extract_config] can only be used once per item",
        )],
    );
}
