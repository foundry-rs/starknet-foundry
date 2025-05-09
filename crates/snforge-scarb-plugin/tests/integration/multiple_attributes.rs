use crate::utils::{assert_diagnostics, assert_output};
use cairo_lang_macro::{quote, TokenStream};
use snforge_scarb_plugin::attributes::fuzzer::wrapper::fuzzer_wrapper;
use snforge_scarb_plugin::attributes::fuzzer::{fuzzer, fuzzer_config};
use snforge_scarb_plugin::attributes::{available_gas::available_gas, fork::fork, test::test};

#[test]
fn works_with_few_attributes() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

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
    let args = quote!((l1_gas: 1, l1_data_gas: 2, l2_gas: 3));

    let result = available_gas(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::AvailableGasConfig::MaxResourceBounds(
                        snforge_std::_internals::config_types::AvailableResourceBoundsConfig {
                            l1_gas: 0x1,
                            l1_data_gas: 0x2,
                            l2_gas: 0x3
                        }
                    )
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());

                    return;
                }
            }
        ",
    );

    let item = result.token_stream;
    let args = quote!(("test"));

    let result = fork(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::AvailableGasConfig::MaxResourceBounds(
                        snforge_std::_internals::config_types::AvailableResourceBoundsConfig {
                            l1_gas: 0x1,
                            l1_data_gas: 0x2,
                            l2_gas: 0x3
                        }
                    )
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());

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
fn works_with_fuzzer() {
    let item = quote!(
        fn empty_fn() {}
    );
    let args = TokenStream::empty();

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
    let args = quote!((runs: 123, seed: 321));

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

#[test]
#[expect(clippy::too_many_lines)]
fn works_with_fuzzer_config_wrapper() {
    let item = quote!(
        fn empty_fn(f: felt252) {}
    );
    let args = quote!((l2_gas: 999));

    let result = available_gas(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            fn empty_fn(f: felt252) {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::AvailableGasConfig::MaxResourceBounds(
                        snforge_std::_internals::config_types::AvailableResourceBoundsConfig {
                            l1_gas: 0xffffffffffffffff,
                            l1_data_gas: 0xffffffffffffffff,
                            l2_gas: 0x3e7
                        }
                    )
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());

                    return;
                }
            }
        ",
    );

    // Cannot apply `test` attribute here as it would cause an error
    // due to the function having a parameter
    let item = result.token_stream;
    let item = quote!(
        #[snforge_internal_test_executable]
        #[__internal_config_statement]
        #item
    );
    let args = quote!((runs: 123, seed: 321));

    let result = fuzzer_config(args, item);
    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn(f: felt252) {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::AvailableGasConfig::MaxResourceBounds(
                        snforge_std::_internals::config_types::AvailableResourceBoundsConfig {
                            l1_gas: 0xffffffffffffffff,
                            l1_data_gas: 0xffffffffffffffff,
                            l2_gas: 0x3e7
                        }
                    )
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());

                    let mut data = array![];

                    snforge_std::_internals::config_types::FuzzerConfig {
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
    let args = TokenStream::empty();

    let result = fuzzer_wrapper(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
            #[snforge_internal_test_executable]
            #[__internal_config_statement]
            fn empty_fn() {
                if snforge_std::_internals::is_config_run() {
                    let mut data = array![];

                    snforge_std::_internals::config_types::AvailableGasConfig::MaxResourceBounds(
                        snforge_std::_internals::config_types::AvailableResourceBoundsConfig {
                            l1_gas: 0xffffffffffffffff,
                            l1_data_gas: 0xffffffffffffffff,
                            l2_gas: 0x3e7
                        }
                    )
                    .serialize(ref data);

                    starknet::testing::cheatcode::<'set_config_available_gas'>(data.span());

                    let mut data = array![];

                    snforge_std::_internals::config_types::FuzzerConfig {
                        seed: Option::Some(0x141),
                        runs: Option::Some(0x7b)
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
