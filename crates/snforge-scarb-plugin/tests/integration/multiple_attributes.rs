use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN, FN_WITH_SINGLE_FELT252_PARAM};
use cairo_lang_macro::TokenStream;
use snforge_scarb_plugin::attributes::fuzzer::wrapper::fuzzer_wrapper;
use snforge_scarb_plugin::attributes::fuzzer::{fuzzer, fuzzer_config};
use snforge_scarb_plugin::attributes::{available_gas::available_gas, fork::fork, test::test};

#[test]
fn works_with_few_attributes() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(String::new());

    let result = test(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn(){}
        ",
    );

    let item = result.token_stream;
    let args = TokenStream::new("(123)".into());

    let result = available_gas(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::AvailableGasConfig {
                        gas: 0x7b
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());

                    return; 
                }
            }
        ",
    );

    let item = result.token_stream;
    let args = TokenStream::new(r#"("test")"#.into());

    let result = fork(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::AvailableGasConfig {
                        gas: 0x7b
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());

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
fn works_with_fuzzer() {
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(String::new());

    let result = test(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn(){}
        ",
    );

    let item = result.token_stream;
    let args = TokenStream::new("(runs: 123, seed: 321)".into());

    let result = fuzzer(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            #[__fuzzer_config(runs: 123, seed: 321)]
            #[__fuzzer_wrapper]
            fn empty_fn() {}
        ",
    );
}

#[expect(clippy::too_many_lines)]
#[test]
fn works_with_fuzzer_config_wrapper() {
    let item = TokenStream::new(FN_WITH_SINGLE_FELT252_PARAM.into());
    let args = TokenStream::new("(999)".into());

    let result = available_gas(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn(f: felt252) {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::AvailableGasConfig {
                        gas: 0x3e7
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());

                    return;
                }
            }
        ",
    );

    let item = result.token_stream;
    let args = TokenStream::new(String::new());

    let result = test(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn(f: felt252) {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::AvailableGasConfig {
                        gas: 0x3e7
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());

                    return;
                }
            }
        ",
    );

    let item = result.token_stream;
    let args = TokenStream::new("(runs: 123, seed: 321)".into());

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn(f: felt252) {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::AvailableGasConfig {
                        gas: 0x3e7
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());

                    let mut data = array![];

                    snforge_std::_config_types::FuzzerConfig {
                        seed: Option::Some(0x141),
                        runs: Option::Some(0x7b)
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
        r"
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn() {
                if snforge_std::_internals::_is_config_run() {
                    let mut data = array![];

                    snforge_std::_config_types::AvailableGasConfig {
                        gas: 0x3e7
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());

                    let mut data = array![];

                    snforge_std::_config_types::FuzzerConfig {
                        seed: Option::Some(0x141),
                        runs: Option::Some(0x7b)
                    }
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_fuzzer'>(data.span());

                    empty_fn_actual_body(snforge_std::fuzzable::Fuzzable::blank());

                    return;
                }
                let f: felt252 = snforge_std::fuzzable::Fuzzable::generate();
                snforge_std::_internals::_save_fuzzer_arg(@f);
                empty_fn_actual_body(f);
            }
            #[__internal_config_statement]
            fn empty_fn_actual_body(f: felt252) {
            }
        ",
    );
}
