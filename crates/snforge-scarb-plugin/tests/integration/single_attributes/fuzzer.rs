use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN, FN_WITH_SINGLE_FELT252_PARAM};
use cairo_lang_macro::{Diagnostic, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::fuzzer::wrapper::fuzzer_wrapper;
use snforge_scarb_plugin::attributes::fuzzer::{fuzzer, fuzzer_config};

#[test]
fn work_without_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(String::new());

    let result = fuzzer(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[__fuzzer_config]
            #[__fuzzer_wrapper]
            fn empty_fn() {}
        ",
    );
}

#[test]
fn work_with_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(runs: 655, seed: 32872357)".into());

    let result = fuzzer(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[__fuzzer_config(runs: 655, seed: 32872357)]
            #[__fuzzer_wrapper]
            fn empty_fn() {}
        ",
    );
}

#[test]
fn config_works_with_runs_only() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(runs: 655)".into());

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::FuzzerConfig {
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
fn config_works_with_seed_only() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(seed: 655)".into());

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::FuzzerConfig {
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
fn config_works_with_both_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(runs: 655, seed: 32872357)".into());

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::FuzzerConfig {
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
fn config_wrapper_work_without_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(String::new());

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::FuzzerConfig {
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

    let item = result.token_stream;
    let args = TokenStream::new(String::new());

    let result = fuzzer_wrapper(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::FuzzerConfig {
                        seed: Option::None,
                        runs: Option::None
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_fuzzer'>(data.span());

                    empty_fn_actual_body();

                    return;
                }
                empty_fn_actual_body();
            }

            #[__internal_config_statement]
            fn empty_fn_actual_body() {
            }
        ",
    );
}

#[test]
fn config_wrapper_work_with_both_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(runs: 655, seed: 32872357)".into());

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::FuzzerConfig {
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

    let item = result.token_stream;
    let args = TokenStream::new(String::new());

    let result = fuzzer_wrapper(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::FuzzerConfig {
                        seed: Option::Some(0x1f597a5),
                        runs: Option::Some(0x28f)
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_fuzzer'>(data.span());

                    empty_fn_actual_body();

                    return;
                }
                empty_fn_actual_body();
            }

            #[__internal_config_statement]
            fn empty_fn_actual_body() {
            }
        ",
    );
}

#[test]
fn config_wrapper_work_with_fn_with_param() {
    let item = TokenStream::new(FN_WITH_SINGLE_FELT252_PARAM.into());
    let args = TokenStream::new(String::new());

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn(f: felt252) {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::FuzzerConfig {
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

    let item = result.token_stream;
    let args = TokenStream::new(String::new());

    let result = fuzzer_wrapper(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::FuzzerConfig {
                        seed: Option::None,
                        runs: Option::None
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_fuzzer'>(data.span());

                    empty_fn_actual_body(snforge_std::fuzzable::Fuzzable::blank());

                    return;
                }
                let f: felt252 = snforge_std::fuzzable::Fuzzable::generate();
                empty_fn_actual_body(f);
            }
            #[__internal_config_statement]
            fn empty_fn_actual_body(f: felt252) {
            }
        ",
    );
}

#[test]
fn wrapper_handle_attributes() {
    let item = TokenStream::new(formatdoc!(
        "
            #[available_gas(1)]
            #[test]
            {EMPTY_FN}
        "
    ));
    let args = TokenStream::new(String::new());

    let result = fuzzer_wrapper(args, item);

    assert_output(
        &result,
        "
            #[test]
            fn empty_fn() { 
                if snforge_std::_internals::_is_config_run() {
                    empty_fn_actual_body();

                    return; 
                } 
                empty_fn_actual_body(); 
            }

            #[available_gas(1)]
            #[__internal_config_statement]
            fn empty_fn_actual_body() {
            }
        ",
    );
}

#[test]
fn fail_with_invalid_args() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(seed: '655')".into());

    let result = fuzzer_config(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[fuzzer] <seed> should be number literal",
        )],
    );
}

#[test]
fn fail_with_unnamed_arg() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new("(123)".into());

    let result = fuzzer_config(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[fuzzer] can be used with named attributes only",
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
