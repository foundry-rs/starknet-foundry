use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN};
use cairo_lang_macro::{Diagnostic, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::fuzzer::fuzzer;

#[test]
fn work_without_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(String::new());

    let result = fuzzer(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::_config_types::FuzzerConfig {
                        seed: Option::None,
                        runs: Option::None
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_fuzzer'>(data.span());

                    return;
                }
            }
        ",
    );
}

#[test]
fn work_with_both_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(runs: 655, seed: 32872357)".into());

    let result = fuzzer(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::_config_types::FuzzerConfig {
                        seed: Option::Some(0x1f597a5),
                        runs: Option::Some(0x28f)
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_fuzzer'>(data.span());

                    return;
                }
            }
        ",
    );
}

#[test]
fn work_with_runs_only() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(runs: 655)".into());

    let result = fuzzer(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::_config_types::FuzzerConfig {
                        seed: Option::None,
                        runs: Option::Some(0x28f)
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_fuzzer'>(data.span());

                    return;
                }
            }
        ",
    );
}

#[test]
fn work_with_seed_only() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(seed: 655)".into());

    let result = fuzzer(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::_config_types::FuzzerConfig {
                        seed: Option::Some(0x28f),
                        runs: Option::None
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_fuzzer'>(data.span());

                    return;
                }
            }
        ",
    );
}

#[test]
fn fail_with_invalid_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(seed: '655')".into());

    let result = fuzzer(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[fuzzer] <seed> should be number literal",
        )],
    );
}

#[test]
fn is_used_once() {
    let item = TokenStream::new(formatdoc!(
        "
            #[fuzzer]
            {EMPTY_FN}
        "
    ));
    let args = TokenStream::new(String::new());

    let result = fuzzer(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[fuzzer] can only be used once per item",
        )],
    );
}
