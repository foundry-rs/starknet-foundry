use crate::utils::{assert_diagnostics, assert_output};
use cairo_lang_macro::{quote, Diagnostic};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::fork::fork;

#[test]
fn fails_without_block() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((url: "invalid url"));

    let result = fork(args, item);

    assert_diagnostics(
        &result,
        &[
           Diagnostic::error(formatdoc!(
                "
                    All options failed
                    - variant: exactly one of <block_hash> | <block_number> | <block_tag> should be specified, got 0
                    - variant: #[fork] expected arguments: 1, got: 0
                    - variant: #[fork] can be used with unnamed arguments only
                    Resolve at least one of them
                "
            ))
        ],
    );
}

#[test]
fn fails_without_url() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((block_number: 23));

    let result = fork(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "
                All options failed
                - variant: <url> argument is missing
                - variant: #[fork] expected arguments: 1, got: 0
                - variant: #[fork] can be used with unnamed arguments only
                Resolve at least one of them
            "
        ))],
    );
}

#[test]
fn fails_without_args() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!(());

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
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((url: "invalid url", block_number: 23));

    let result = fork(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(formatdoc!(
            "
                All options failed
                - variant: #[fork] <url> is not a valid url
                - variant: #[fork] expected arguments: 1, got: 0
                - variant: #[fork] can be used with unnamed arguments only
                Resolve at least one of them
            "
        ))],
    );
}

#[test]
fn accepts_string() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!(("test"));

    let result = fork(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {

                    let mut data = array![];

                    snforge_std::_internals::config_types::ForkConfig::Named("test")
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
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((url: "http://example.com", block_number: 23));

    let result = fork(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {

                    let mut data = array![];

                    snforge_std::_internals::config_types::ForkConfig::Inline(
                        snforge_std::_internals::config_types::InlineForkConfig {
                            url: "http://example.com/",
                            block: snforge_std::_internals::config_types::BlockId::BlockNumber(0x17)
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
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!(("MAINNET", block_number: 23));

    let result = fork(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {

                    let mut data = array![];

                    snforge_std::_internals::config_types::ForkConfig::Overridden(
                        snforge_std::_internals::config_types::OverriddenForkConfig {
                            block: snforge_std::_internals::config_types::BlockId::BlockNumber(0x17),
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
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((block_number: 23, "MAINNET"));

    let result = fork(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {

                    let mut data = array![];

                    snforge_std::_internals::config_types::ForkConfig::Overridden(
                        snforge_std::_internals::config_types::OverriddenForkConfig {
                            block: snforge_std::_internals::config_types::BlockId::BlockNumber(0x17),
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
    let item = quote!(
        #[fork]
        fn empty_fn() {}
    );
    let args = quote!(("name"));

    let result = fork(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error("#[fork] can only be used once per item")],
    );
}
