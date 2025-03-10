use crate::utils::{assert_diagnostics, assert_output};
use cairo_lang_macro::{quote, Diagnostic, TextSpan, Token, TokenStream, TokenTree};
use snforge_scarb_plugin::attributes::fuzzer::wrapper::fuzzer_wrapper;
use snforge_scarb_plugin::attributes::fuzzer::{fuzzer, fuzzer_config};

#[test]
fn work_without_args() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

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
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((runs: 655, seed: 32872357));

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
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((runs: 655));

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::FuzzerConfig {
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
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((seed: 655));

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::FuzzerConfig {
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
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((runs: 655, seed: 32872357));

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::FuzzerConfig {
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
    let item = quote!(
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::FuzzerConfig {
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
    let args = TokenStream::empty();

    let result = fuzzer_wrapper(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::FuzzerConfig {
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
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((runs: 655, seed: 32872357));

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::FuzzerConfig {
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
    let args = TokenStream::empty();

    let result = fuzzer_wrapper(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::FuzzerConfig {
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
fn config_wrapper_work_with_fn_with_single_param() {
    let item = quote!(
        fn empty_fn(f: felt252) {}
    );
    let args = TokenStream::empty();

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn(f: felt252) {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::FuzzerConfig {
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
    let args = TokenStream::empty();

    let result = fuzzer_wrapper(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::FuzzerConfig {
                        seed: Option::None,
                        runs: Option::None
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_fuzzer'>(data.span());

                    empty_fn_actual_body(snforge_std::fuzzable::Fuzzable::blank());

                    return;
                }
                let f: felt252 = snforge_std::fuzzable::Fuzzable::generate();
                snforge_std::_internals::save_fuzzer_arg(@f);
                empty_fn_actual_body(f);
            }
            #[__internal_config_statement]
            fn empty_fn_actual_body(f: felt252) {
            }
        ",
    );
}

#[test]
fn config_wrapper_work_with_fn_with_params() {
    let item = quote!(
        fn empty_fn(f: felt252, u: u32) {}
    );
    let args = TokenStream::empty();

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn(f: felt252, u: u32) {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::FuzzerConfig {
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
    let args = TokenStream::empty();

    let result = fuzzer_wrapper(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::FuzzerConfig {
                        seed: Option::None,
                        runs: Option::None
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_fuzzer'>(data.span());

                    empty_fn_actual_body(snforge_std::fuzzable::Fuzzable::blank(), snforge_std::fuzzable::Fuzzable::blank());

                    return;
                }
                let f: felt252 = snforge_std::fuzzable::Fuzzable::generate();
                snforge_std::_internals::save_fuzzer_arg(@f);
                let u: u32 = snforge_std::fuzzable::Fuzzable::generate();
                snforge_std::_internals::save_fuzzer_arg(@u);
                empty_fn_actual_body(f, u);
            }
            #[__internal_config_statement]
            fn empty_fn_actual_body(f: felt252, u: u32) {
            }
        ",
    );
}

#[test]
fn wrapper_handle_attributes() {
    let item = quote!(
        #[available_gas(l2_gas: 40000)]
        #[test]
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = fuzzer_wrapper(args, item);

    assert_output(
        &result,
        "
            #[test]
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    empty_fn_actual_body();

                    return;
                }
                empty_fn_actual_body();
            }

            #[available_gas(l2_gas: 40000)]
            #[__internal_config_statement]
            fn empty_fn_actual_body() {
            }
        ",
    );
}

#[test]
fn fail_with_invalid_args() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = TokenStream::new(vec![TokenTree::Ident(Token::new(
        "(seed: '655')",
        TextSpan::call_site(),
    ))]);

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
    let item = quote!(
        fn empty_fn() {}
    );
    let args = quote!((123));

    let result = fuzzer_config(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[fuzzer] can be used with named arguments only",
        )],
    );
}

#[test]
fn is_used_once() {
    let item = quote!(
        #[fuzzer]
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

    let result = fuzzer(args, item);

    assert_diagnostics(
        &result,
        &[Diagnostic::error(
            "#[fuzzer] can only be used once per item",
        )],
    );
}
