use crate::utils::{assert_diagnostics, empty_function, format_output};
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

    insta::with_settings!({ snapshot_suffix => "after_test" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    let item = get_function(&result.token_stream, "empty_fn", false);
    let args = quote!((l1_gas: 1, l1_data_gas: 2, l2_gas: 3));

    let result = available_gas(args, item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_available_gas" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    let item = result.token_stream;
    let args = quote!(("test"));

    let result = fork(args, item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fork" }, {
        insta::assert_snapshot!(format_output(&result));
    });
}

#[test]
fn works_with_fuzzer() {
    let args = TokenStream::empty();

    let result = test(args, empty_function());

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_test" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    let item = get_function(&result.token_stream, "empty_fn", false);
    let args = quote!((runs: 123, seed: 321));

    let result = fuzzer(args, item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer" }, {
        insta::assert_snapshot!(format_output(&result));
    });
}

#[test]
fn works_with_fuzzer_before_test() {
    let item = quote!(
        fn empty_fn(f: felt252) {}
    );
    let fuzzer_args = quote!((runs: 123, seed: 321));
    let fuzzer_res = fuzzer(fuzzer_args, item);
    assert_diagnostics(&fuzzer_res, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer" }, {
        insta::assert_snapshot!(format_output(&fuzzer_res));
    });

    let test_args = TokenStream::empty();
    let item = get_function(&fuzzer_res.token_stream, "empty_fn", false);
    let result = test(test_args, item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_test" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    // We need to remove `#[__fuzzer_wrapper]` to be able to call `fuzzer_wrapper()` again
    let item = get_function(&result.token_stream, "empty_fn", true);
    let item = quote!(
        #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
        #[snforge_internal_test_executable]
        #item

        #[__fuzzer_config(runs: 123, seed: 321)]
        #[__internal_config_statement]
        fn empty_fn() {}
    );
    let result = fuzzer_wrapper(TokenStream::empty(), item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer_wrapper" }, {
        insta::assert_snapshot!(format_output(&result));
    });
}

#[test]
fn works_with_fuzzer_config_wrapper() {
    let item = quote!(
        fn empty_fn(f: felt252) {}
    );
    let args = quote!((l2_gas: 999));

    let result = available_gas(args, item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_available_gas" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    // Append `#[fuzzer]` so we can use `test()`
    let mut item = TokenStream::new(vec![create_single_token("#[fuzzer]")]);
    item.extend(result.token_stream);

    let result = test(TokenStream::empty(), item);
    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_test" }, {
        insta::assert_snapshot!(format_output(&result));
    });

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

    insta::with_settings!({ snapshot_suffix => "after_fuzzer_config" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    let item = result.token_stream;
    let args = TokenStream::empty();

    let result = fuzzer_wrapper(args, item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer_wrapper" }, {
        insta::assert_snapshot!(format_output(&result));
    });
}

use snforge_scarb_plugin::attributes::test_case::test_case;

#[test]
fn works_with_test_fuzzer_and_test_case() {
    // Ad 1. We must add `#[test_case]` first so `#[test]` will not throw
    // diagnostic error "function with parameters must have #[fuzzer] or #[test_case] attribute".
    // It will be later removed (Ad 2.).
    let item = quote!(
        #[test_case(name: "one_and_two", 1, 2, 3)]
        fn test_add(x: i128, y: i128, expected: i128) {}
    );

    let result = test(TokenStream::empty(), item.clone());
    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_test" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    let item = get_function(&result.token_stream, "test_add", false);
    let result = fuzzer(TokenStream::empty(), item);
    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    // Ad 2. Now, we need to remove `#[test_case]` before calling `test_case()`.
    let item = get_function(&result.token_stream, "test_add", true);
    let item = quote! {
        #[__fuzzer_config]
        #[__fuzzer_wrapper]
        #item
    };

    let args = quote!((name: "one_and_two", 1, 2, 3));
    let result = test_case(args, item);
    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_test_case" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    let item = get_function(&result.token_stream, "test_add", true);
    let item = quote!(
        #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
        #[snforge_internal_test_executable]
        fn test_add_one_and_two(mut _data: Span<felt252>) -> Span::<felt252> {
            core::internal::require_implicit::<System>();
            core::internal::revoke_ap_tracking();
            core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), "Out of gas");
            core::option::OptionTraitImpl::expect(
                core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), "Out of gas",
            );
            test_add(1, 2, 3);
            let mut arr = ArrayTrait::new();
            core::array::ArrayTrait::span(@arr)
        }

        #[__fuzzer_wrapper]
        #item
    );
    let result = fuzzer_config(TokenStream::empty(), item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer_config" }, {
        insta::assert_snapshot!(format_output(&result));
    });

    let item = get_function(&result.token_stream, "test_add_one_and_two", false);
    let result = fuzzer_wrapper(TokenStream::empty(), item);

    assert_diagnostics(&result, &[]);

    insta::with_settings!({ snapshot_suffix => "after_fuzzer_wrapper" }, {
        insta::assert_snapshot!(format_output(&result));
    });
}
