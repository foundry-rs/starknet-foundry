use super::{AttributeInfo, ErrorExt, internal_config_statement::InternalConfigStatementCollector};
use crate::attributes::test_case::TestCaseCollector;
use crate::common::no_fuzzer_attribute;
use crate::utils::create_single_token;
use crate::{
    args::Arguments,
    common::{into_proc_macro_result, with_parsed_values},
    format_ident,
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode, ast::FunctionWithBody};
use std::env::{self, VarError};

pub struct TestCollector;

impl AttributeInfo for TestCollector {
    const ATTR_NAME: &'static str = "test";
}

#[must_use]
pub fn test(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, |args, item, warns| {
        with_parsed_values::<TestCollector>(args, item, warns, test_internal)
    })
}

#[expect(clippy::ptr_arg)]
#[expect(clippy::needless_pass_by_value)]
fn test_internal(
    db: &SimpleParserDatabase,
    func: &FunctionWithBody,
    _args_db: &SimpleParserDatabase,
    args: Arguments,
    _warns: &mut Vec<Diagnostic>,
) -> Result<TokenStream, Diagnostics> {
    args.assert_is_empty::<TestCollector>()?;
    ensure_parameters_only_with_fuzzer_attribute(db, func)?;
    assert_no_test_case_attribute(db, func)?;

    let internal_config = create_single_token(InternalConfigStatementCollector::ATTR_NAME);

    let func_item = func.as_syntax_node();
    let func_item = SyntaxNodeWithDb::new(&func_item, db);

    let name = func.declaration(db).name(db).text(db).to_string();

    let test_filter = get_forge_test_filter().ok();

    let should_run_test = match test_filter {
        Some(ref filter) => name.contains(filter),
        None => true,
    };

    let name = func.declaration(db).name(db).as_syntax_node();
    let name = SyntaxNodeWithDb::new(&name, db);

    let signature = func.declaration(db).signature(db).as_syntax_node();
    let signature = SyntaxNodeWithDb::new(&signature, db);
    let signature = quote! { #signature };

    let body = func.body(db).as_syntax_node();
    let body = SyntaxNodeWithDb::new(&body, db);

    let attributes = func.attributes(db).as_syntax_node();
    let attributes = SyntaxNodeWithDb::new(&attributes, db);

    let name_return_wrapper =
        format_ident!("{}_return_wrapper", func.declaration(db).name(db).text(db));

    let mut return_wrapper = TokenStream::new(vec![name_return_wrapper.clone()]);
    return_wrapper.extend(signature);

    let out_of_gas = create_single_token("'Out of gas'");

    if should_run_test {
        Ok(quote!(
            #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
            #[snforge_internal_test_executable]
            fn #name(mut _data: Span<felt252>) -> Span::<felt252> {
                core::internal::require_implicit::<System>();
                core::internal::revoke_ap_tracking();
                core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), #out_of_gas);

                core::option::OptionTraitImpl::expect(
                    core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), #out_of_gas
                );
                #name_return_wrapper();

                let mut arr = ArrayTrait::new();
                core::array::ArrayTrait::span(@arr)
            }

            #attributes
            #[#internal_config]
            fn #return_wrapper
            #body
        ))
    } else {
        Ok(quote!(
            #[#internal_config]
            #func_item
        ))
    }
}

fn get_forge_test_filter() -> Result<String, VarError> {
    env::var("SNFORGE_TEST_FILTER")
}

fn ensure_parameters_only_with_fuzzer_attribute(
    db: &SimpleParserDatabase,
    func: &FunctionWithBody,
) -> Result<(), Diagnostic> {
    if has_parameters(db, func) && no_fuzzer_attribute(db, func) {
        Err(TestCollector::error(
            "function with parameters must have #[fuzzer] attribute",
        ))?;
    }

    Ok(())
}

fn has_parameters(db: &SimpleParserDatabase, func: &FunctionWithBody) -> bool {
    func.declaration(db)
        .signature(db)
        .parameters(db)
        .elements(db)
        .len()
        != 0
}

fn assert_no_test_case_attribute(
    db: &SimpleParserDatabase,
    func: &FunctionWithBody,
) -> Result<(), Diagnostic> {
    let test_case_attr = func
        .attributes(db)
        .find_attr(db, TestCaseCollector::ATTR_NAME);
    Ok(if test_case_attr.is_some() {
        Err(TestCollector::error(
            "#[test] attribute cannot be used with #[test_case(...)]",
        ))?;
    })
}
