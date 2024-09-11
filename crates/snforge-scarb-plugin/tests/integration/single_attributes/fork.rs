use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN};
use cairo_lang_macro::{Diagnostic, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::fork::fork;

#[test]
fn fails_without_block() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(r#"(url: "invalid url")"#.into());

    let result = fork(args, item);

    assert_diagnostics(
        &result,
        &[
           Diagnostic::error(formatdoc!(
                "
                    Both options failed
                    First variant: exactly one of <block_hash> | <block_number> | <block_tag> should be specified, got 0
                    Second variant: #[fork] can be used with unnamed attributes only
                    Resolve at least one of them
                "
            ))
        ],
    );
}

#[test]
fn fails_without_url() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(block_number: 23)".into());

    let result = fork(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "
                Both options failed
                First variant: <url> argument is missing
                Second variant: #[fork] can be used with unnamed attributes only
                Resolve at least one of them
            "
        ))],
    );
}

#[test]
fn fails_without_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("()".into());

    let result = fork(args, item);

    assert_diagnostics(
        &result,
        &[
          Diagnostic::error(formatdoc!(
            "
                Both options failed
                First variant: exactly one of <block_hash> | <block_number> | <block_tag> should be specified, got 0
                Second variant: #[fork] expected 1 arguments, got: 0
                Resolve at least one of them
            "))
        ],
    );
}

#[test]
fn fails_with_invalid_url() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(r#"(url: "invalid url", block_number: 23)"#.into());

    let result = fork(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "
                Both options failed
                First variant: #[fork] <url> is not a valid url
                Second variant: #[fork] can be used with unnamed attributes only
                Resolve at least one of them
            "
        ))],
    );
}

#[test]
fn accepts_string() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(r#"("test")"#.into());

    let result = fork(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            fn empty_fn() {
                if snforge_std::_cheatcode::_is_config_run() {

                    let mut data = array![];

                    snforge_std::_config_types::ForkConfig::Named("test")
                        .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_fork'>(data.span());

                    return;
                }
            }
        "#,
    );
}

#[test]
fn accepts_inline_config() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(r#"(url: "http://example.com", block_number: 23)"#.into());

    let result = fork(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            fn empty_fn() {
                if snforge_std::_cheatcode::_is_config_run() {

                    let mut data = array![];

                    snforge_std::_config_types::ForkConfig::Inline(
                        snforge_std::_config_types::InlineForkConfig {
                            url: "http://example.com/",
                            block: snforge_std::_config_types::BlockId::BlockNumber(0x17)
                        }
                    )
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_fork'>(data.span());

                    return;
                }
            }
        "#,
    );
}

#[test]
fn is_used_once() {
    let item = TokenStream::new(formatdoc!(
        "
            #[fork]
            {EMPTY_FN}
        "
    ));
    let args = TokenStream::new(r#"("name")"#.into());

    let result = fork(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error("#[fork] can only be used once per item")],
    );
}
