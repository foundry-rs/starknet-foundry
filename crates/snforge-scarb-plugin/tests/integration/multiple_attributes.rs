use crate::utils::{assert_diagnostics, assert_output, EMPTY_FN, FN_WITH_SINGLE_FELT252_PARAM};
use cairo_lang_macro::{ProcMacroResult, TokenStream};
use indoc::formatdoc;
use snforge_scarb_plugin::attributes::fuzzer::wrapper::fuzzer_wrapper;
use snforge_scarb_plugin::attributes::fuzzer::{fuzzer, fuzzer_config};
use snforge_scarb_plugin::attributes::{available_gas::available_gas, fork::fork, test::test};

fn last_n_lines(result: &ProcMacroResult, n_lines: usize) -> TokenStream {
    let lines = result.token_stream.to_string();
    let lines = lines
        .lines()
        .skip(lines.lines().count() - n_lines)
        .collect::<String>();
    TokenStream::new(lines)
}

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
            #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
            fn empty_fn(mut _data: Span<felt252>) -> Span::<felt252> {
                core::internal::require_implicit::<System>();
                core::internal::revoke_ap_tracking();
                core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), 'Out of gas');

                core::option::OptionTraitImpl::expect(
                    core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), 'Out of gas',
                );
                empty_fn_return_wrapper();

                let mut arr = ArrayTrait::new();
                core::array::ArrayTrait::span(@arr)
            }

            #[__internal_config_statement]
            fn empty_fn_return_wrapper() {

            }
        ",
    );

    let item = last_n_lines(&result, 4);
    let args = TokenStream::new("(l1_gas: 1, l1_data_gas: 2, l2_gas: 3)".into());

    let result = available_gas(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[__internal_config_statement]
            fn empty_fn_return_wrapper() {
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
    let args = TokenStream::new(r#"("test")"#.into());

    let result = fork(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
            #[__internal_config_statement]
            fn empty_fn_return_wrapper() {
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
    let item = TokenStream::new(EMPTY_FN.into());
    let args = TokenStream::new(String::new());

    let result = test(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
           #[snforge_internal_test_executable]
            #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
            fn empty_fn(mut _data: Span<felt252>) -> Span::<felt252> {
                core::internal::require_implicit::<System>();
                core::internal::revoke_ap_tracking();
                core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), 'Out of gas');

                core::option::OptionTraitImpl::expect(
                    core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), 'Out of gas',
                );
                empty_fn_return_wrapper();

                let mut arr = ArrayTrait::new();
                core::array::ArrayTrait::span(@arr)
            }

            #[__internal_config_statement]
            fn empty_fn_return_wrapper() {

            }
        ",
    );

    let item = last_n_lines(&result, 4);
    let args = TokenStream::new("(runs: 123, seed: 321)".into());

    let result = fuzzer(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
            #[__internal_config_statement]
            #[__fuzzer_config(runs: 123, seed: 321)]
            #[__fuzzer_wrapper]
            fn empty_fn_return_wrapper() {}
        ",
    );
}

#[test]
#[expect(clippy::too_many_lines)]
fn works_with_fuzzer_config_wrapper() {
    let item = TokenStream::new(FN_WITH_SINGLE_FELT252_PARAM.into());
    let args = TokenStream::new("(l2_gas: 999)".into());

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

    // Append `#[fuzzer]` so we can use `test()`
    let item = TokenStream::new(formatdoc!(
        r"
        #[fuzzer]
        {}
        ",
        result.token_stream
    ));

    let result = test(TokenStream::new(String::new()), item);
    assert_diagnostics(&result, &[]);
    assert_output(
        &result,
        r"
            #[snforge_internal_test_executable]
            #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
            fn empty_fn(mut _data: Span<felt252>) -> Span::<felt252> {
                core::internal::require_implicit::<System>();
                core::internal::revoke_ap_tracking();
                core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), 'Out of gas');

                core::option::OptionTraitImpl::expect(
                    core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), 'Out of gas',
                );
                empty_fn_return_wrapper();

                let mut arr = ArrayTrait::new();
                core::array::ArrayTrait::span(@arr)
            }

            #[fuzzer]
            #[__internal_config_statement]
            fn empty_fn_return_wrapper(f: felt252) {
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

    // Skip all the lines including `#[fuzzer]` that was appended previously
    let item = last_n_lines(&result, 25);
    let args = TokenStream::new("(runs: 123, seed: 321)".into());

    let result = fuzzer_config(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
            #[__internal_config_statement]
            fn empty_fn_return_wrapper(f: felt252) {
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
    let args = TokenStream::new(String::new());

    let result = fuzzer_wrapper(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
            #[__internal_config_statement]
            fn empty_fn_return_wrapper() {
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

                    empty_fn_return_wrapper_actual_body(snforge_std::fuzzable::Fuzzable::blank());

                    return;
                }
                let f: felt252 = snforge_std::fuzzable::Fuzzable::generate();
                snforge_std::_internals::save_fuzzer_arg(@f);
                empty_fn_return_wrapper_actual_body(f);
            }
            #[__internal_config_statement]
            fn empty_fn_return_wrapper_actual_body(f: felt252) {
            }
        ",
    );
}
