use crate::args::unnamed::UnnamedArgs;
use crate::attributes::test::TestCollector;
use crate::attributes::test_case::name::get_test_case_name;
use crate::attributes::{AttributeInfo, ErrorExt};
use crate::common::{has_fuzzer_attribute, into_proc_macro_result, with_parsed_values};
use crate::utils::SyntaxNodeUtils;
use crate::{create_single_token, format_ident};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::FunctionWithBody;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode};

mod name;

pub struct TestCaseCollector;

impl AttributeInfo for TestCaseCollector {
    const ATTR_NAME: &'static str = "test_case";
}

#[must_use]
pub fn test_case(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, test_case_handler)
}

fn test_case_handler(
    args: &TokenStream,
    item: &TokenStream,
    warns: &mut Vec<Diagnostic>,
) -> Result<TokenStream, Diagnostics> {
    with_parsed_values::<TestCaseCollector>(
        args,
        item,
        warns,
        |func_db, func, args_db, arguments, _warns| {
            let unnamed_args = arguments.unnamed();

            ensure_params_present(func, func_db)?;
            ensure_args_count_valid(func, &unnamed_args, func_db)?;

            let func_name = func.declaration(func_db).name(func_db).text(func_db);
            let case_fn_name = get_test_case_name(&func_name, &arguments, args_db)?;
            let filtered_fn_attrs = get_filtered_func_attributes(func, func_db);

            let signature = func
                .declaration(func_db)
                .signature(func_db)
                .as_syntax_node();
            let signature = SyntaxNodeWithDb::new(&signature, func_db);

            let func_body = func.body(func_db).as_syntax_node();
            let func_body = SyntaxNodeWithDb::new(&func_body, func_db);

            let func_name = format_ident!("{}", func_name);
            let func = quote!(
                #filtered_fn_attrs
                fn #func_name #signature
                #func_body
            );

            let call_args = unnamed_args
                .clone()
                .into_iter()
                .map(|(_, expr)| expr.as_syntax_node().get_text(args_db))
                .collect::<Vec<_>>()
                .join(", ")
                .to_string();
            let call_args = format_ident!("({})", call_args);

            let case_fn_name = format_ident!("{}", case_fn_name);

            let out_of_gas = create_single_token("'Out of gas'");

            Ok(quote!(
                #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
                #[snforge_internal_test_executable]
                fn #case_fn_name(mut _data: Span<felt252>) -> Span::<felt252> {
                    core::internal::require_implicit::<System>();
                    core::internal::revoke_ap_tracking();
                    core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), #out_of_gas);

                    core::option::OptionTraitImpl::expect(
                        core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), #out_of_gas
                    );
                    #func_name #call_args;

                    let mut arr = ArrayTrait::new();
                    core::array::ArrayTrait::span(@arr)
                }

                #func
            ))
        },
    )
}

fn ensure_params_present(
    func: &FunctionWithBody,
    db: &SimpleParserDatabase,
) -> Result<(), Diagnostics> {
    let param_count = func
        .declaration(db)
        .signature(db)
        .parameters(db)
        .elements(db)
        .len();

    if param_count == 0 {
        return Err(Diagnostics::from(TestCaseCollector::error(
            "The function must have at least one parameter to use #[test_case] attribute",
        )));
    }
    Ok(())
}

fn ensure_args_count_valid(
    func: &FunctionWithBody,
    unnamed_args: &UnnamedArgs,
    db: &SimpleParserDatabase,
) -> Result<(), Diagnostics> {
    let param_count = func
        .declaration(db)
        .signature(db)
        .parameters(db)
        .elements(db)
        .len();

    if param_count != unnamed_args.len() {
        return Err(Diagnostics::from(TestCaseCollector::error(format!(
            "Expected {} parameters, but got {}",
            param_count,
            unnamed_args.len()
        ))));
    }
    Ok(())
}

fn get_filtered_func_attributes(
    func: &FunctionWithBody,
    func_db: &SimpleParserDatabase,
) -> TokenStream {
    let attr_list = func.attributes(func_db);
    let has_fuzzer = has_fuzzer_attribute(func_db, func);

    // We do not want to copy the `#[test]` attribute if there is no `#[fuzzer]`
    attr_list
        .elements(func_db)
        .filter(|attr| {
            let test_attr_text = format!("#[{}]", TestCollector::ATTR_NAME);
            let attr_text = attr.as_syntax_node().get_text(func_db);
            let attr_text = attr_text.trim();
            let is_test_attr = attr_text == test_attr_text;

            !is_test_attr || has_fuzzer
        })
        .map(|attr| attr.to_token_stream(func_db))
        .fold(TokenStream::empty(), |mut acc, token| {
            acc.extend(token);
            acc
        })
}
