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
                    All options failed
                    - variant: exactly one of <block_hash> | <block_number> | <block_tag> should be specified, got 0
                    - variant: #[fork] expected arguments: 1, got: 0
                    - variant: #[fork] can be used with unnamed attributes only
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
                All options failed
                - variant: <url> argument is missing
                - variant: #[fork] expected arguments: 1, got: 0
                - variant: #[fork] can be used with unnamed attributes only
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
        &[Diagnostic::warn("#[fork] used with empty argument list. Either remove () or specify some arguments"),
            Diagnostic::error(formatdoc!(
            "
                All options failed
                - variant: exactly one of <block_hash> | <block_number> | <block_tag> should be specified, got 0
                - variant: #[fork] expected arguments: 1, got: 0
                - variant: #[fork] expected arguments: 1, got: 0
                Resolve at least one of them
            "
        ))],
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
                All options failed
                - variant: #[fork] <url> is not a valid url
                - variant: #[fork] expected arguments: 1, got: 0
                - variant: #[fork] can be used with unnamed attributes only
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
                if snforge_std::_internals::_is_config_run() {

                    let mut data = array![];

                    snforge_std::_internals::_config_types::ForkConfig::Named("test")
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
                if snforge_std::_internals::_is_config_run() {

                    let mut data = array![];

                    snforge_std::_internals::_config_types::ForkConfig::Inline(
                        snforge_std::_internals::_config_types::InlineForkConfig {
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
fn overriding_config_name_first() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(r#"("MAINNET", block_number: 23)"#.into());

    let result = fork(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {

                    let mut data = array![];

                    snforge_std::_internals::_config_types::ForkConfig::Overridden(
                        snforge_std::_internals::_config_types::OverriddenForkConfig {
                            block: snforge_std::_internals::_config_types::BlockId::BlockNumber(0x17),
                            name: "MAINNET"
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
fn overriding_config_name_second() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(r#"(block_number: 23, "MAINNET")"#.into());

    let result = fork(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {

                    let mut data = array![];

                    snforge_std::_internals::_config_types::ForkConfig::Overridden(
                        snforge_std::_internals::_config_types::OverriddenForkConfig {
                            block: snforge_std::_internals::_config_types::BlockId::BlockNumber(0x17),
                            name: "MAINNET"
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
