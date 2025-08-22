use crate::utils::{assert_diagnostics, assert_output, empty_function};
use cairo_lang_macro::{TokenStream, TokenTree, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{ModuleItem, SyntaxFile};
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode};
use snforge_scarb_plugin::attributes::fuzzer::wrapper::fuzzer_wrapper;
use snforge_scarb_plugin::attributes::fuzzer::{fuzzer, fuzzer_config};
use snforge_scarb_plugin::attributes::{available_gas::available_gas, fork::fork, test::test};
use snforge_scarb_plugin::create_single_token;

fn get_function(token_stream: &TokenStream, function_name: &str, skip_args: bool) -> TokenStream {
    let db = SimpleParserDatabase::default();
    let (parsed_node, _diagnostics) = db.parse_token_stream(token_stream);
    let syntax_file = SyntaxFile::from_syntax_node(&db, parsed_node);
    let function = syntax_file
        .items(&db)
        .elements(&db)
        .find_map(|e| {
            if let ModuleItem::FreeFunction(free_function) = e {
                if free_function.declaration(&db).name(&db).text(&db) == function_name {
                    Some(free_function.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap();

    let vis = function.visibility(&db).as_syntax_node();
    let vis = SyntaxNodeWithDb::new(&vis, &db);

    let signature = function.declaration(&db).as_syntax_node();
    let signature = SyntaxNodeWithDb::new(&signature, &db);

    let body = function.body(&db).as_syntax_node();
    let body = SyntaxNodeWithDb::new(&body, &db);

    let attrs = function.attributes(&db).as_syntax_node();
    let attrs = SyntaxNodeWithDb::new(&attrs, &db);

    let mut token_stream = if skip_args {
        quote! {
            #vis #signature
            #body
        }
    } else {
        quote! {
            #attrs
            #vis #signature
            #body
        }
    };

    match &mut token_stream.tokens[0] {
        TokenTree::Ident(ident) => {
            ident.span.start = 0;
        }
    }

    token_stream
}

#[test]
fn works_with_few_attributes() {
    let args = TokenStream::empty();

    let result = test(args, empty_function());

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
            #[snforge_internal_test_executable]
            fn empty_fn_return_wrapper(mut _data: Span<felt252>) -> Span::<felt252> {
                core::internal::require_implicit::<System>();
                core::internal::revoke_ap_tracking();
                core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), 'Out of gas');

                core::option::OptionTraitImpl::expect(
                    core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), 'Out of gas',
                );
                empty_fn();

                let mut arr = ArrayTrait::new();
                core::array::ArrayTrait::span(@arr)
            }

            #[__internal_config_statement]
            fn empty_fn() {

            }
        ",
    );

    let item = get_function(&result.token_stream, "empty_fn", false);
    let args = quote!((l1_gas: 1, l1_data_gas: 2, l2_gas: 3));

    let result = available_gas(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
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
    println!("YYY");
    let item = result.token_stream;
    let args = quote!(("test"));

    let result = fork(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r#"
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
    let args = TokenStream::empty();

    let result = test(args, empty_function());

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        "
            #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
            #[snforge_internal_test_executable]
            fn empty_fn_return_wrapper(mut _data: Span<felt252>) -> Span::<felt252> {
                core::internal::require_implicit::<System>();
                core::internal::revoke_ap_tracking();
                core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), 'Out of gas');

                core::option::OptionTraitImpl::expect(
                    core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), 'Out of gas',
                );
                empty_fn();

                let mut arr = ArrayTrait::new();
                core::array::ArrayTrait::span(@arr)
            }

            #[__internal_config_statement]
            fn empty_fn() {

            }
        ",
    );

    let item = get_function(&result.token_stream, "empty_fn", false);
    let args = quote!((runs: 123, seed: 321));

    let result = fuzzer(args, item);

    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
            #[__fuzzer_config(runs: 123, seed: 321)]
            #[__fuzzer_wrapper]
            #[__internal_config_statement]
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

    // Append `#[fuzzer]` so we can use `test()`
    let mut item = TokenStream::new(vec![create_single_token("#[fuzzer]")]);
    item.extend(result.token_stream);

    let result = test(TokenStream::empty(), item);
    assert_diagnostics(&result, &[]);
    assert_output(
        &result,
        r"
            #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
            #[snforge_internal_test_executable]
            fn empty_fn_return_wrapper(mut _data: Span<felt252>) -> Span::<felt252> {
                core::internal::require_implicit::<System>();
                core::internal::revoke_ap_tracking();
                core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), 'Out of gas');

                core::option::OptionTraitImpl::expect(
                    core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), 'Out of gas',
                );
                empty_fn__fuzzer_generated();

                let mut arr = ArrayTrait::new();
                core::array::ArrayTrait::span(@arr)
            }

            #[fuzzer]
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

                    return;
                }
            }
    ",
    );

    // Skip all the lines including `#[fuzzer]` that was appended previously
    let item = get_function(&result.token_stream, "empty_fn", true);
    let internal_config_statement =
        TokenStream::new(vec![create_single_token("__internal_config_statement")]);
    let item = quote! {
        #[#internal_config_statement]
        #item
    };
    let args = quote!((runs: 123, seed: 321));

    let result = fuzzer_config(args, item);
    assert_diagnostics(&result, &[]);

    assert_output(
        &result,
        r"
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
            #[__internal_config_statement]
            fn empty_fn__fuzzer_generated() {
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

                    empty_fn(snforge_std::fuzzable::Fuzzable::blank());

                    return;
                }
                let f: felt252 = snforge_std::fuzzable::Fuzzable::generate();
                snforge_std::_internals::save_fuzzer_arg(@f);
                empty_fn(f);
            }
            #[__internal_config_statement]
            fn empty_fn(f: felt252) {
            }
        ",
    );
}
