use super::{AttributeInfo, ErrorExt, internal_config_statement::InternalConfigStatementCollector};
use crate::asserts::assert_is_used_once;
use crate::common::{has_fuzzer_attribute, has_test_case_attribute};
use crate::external_inputs::ExternalInput;
use crate::utils::create_single_token;
use crate::{
    args::Arguments,
    common::{into_proc_macro_result, with_parsed_values},
    format_ident,
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode, ast::FunctionWithBody};

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

#[expect(clippy::needless_pass_by_value)]
fn test_internal(
    db: &SimpleParserDatabase,
    func: &FunctionWithBody,
    _args_db: &SimpleParserDatabase,
    args: Arguments,
    _warns: &mut Vec<Diagnostic>,
) -> Result<TokenStream, Diagnostics> {
    assert_is_used_once::<TestCollector>(db, func)?;
    args.assert_is_empty::<TestCollector>()?;
    ensure_parameters_only_with_fuzzer_or_test_case_attribute(db, func)?;

    let has_test_case = has_test_case_attribute(db, func);
    let has_fuzzer = has_fuzzer_attribute(db, func);

    // If the function has `#[test_case]` attribute and does not have `#[fuzzer]`, we can
    // safely skip code generation from `#[test]`. It will be handled later by `#[test_case]`.
    if has_test_case && !has_fuzzer {
        let func_item = func.as_syntax_node();
        let func_item = SyntaxNodeWithDb::new(&func_item, db);

        return Ok(quote!(
            #func_item
        ));
    }

    let internal_config = create_single_token(InternalConfigStatementCollector::ATTR_NAME);

    let func_item = func.as_syntax_node();
    let func_item = SyntaxNodeWithDb::new(&func_item, db);

    let name = func.declaration(db).name(db).text(db).to_string();

    let test_filter = ExternalInput::get().forge_test_filter;

    let should_run_test = match test_filter {
        Some(ref filter) => name.contains(filter),
        None => true,
    };

    let has_fuzzer = has_fuzzer_attribute(db, func);

    // If there is `#[fuzzer]` attribute, called function is suffixed with `__snforge_internal_fuzzer_generated`
    // `#[__fuzzer_wrapper]` is responsible for adding this suffix.
    let called_func_ident = if has_fuzzer {
        format_ident!("{name}__snforge_internal_fuzzer_generated")
    } else {
        format_ident!("{name}")
    };
    let called_func = TokenStream::new(vec![called_func_ident]);

    let signature = func.declaration(db).signature(db).as_syntax_node();
    let signature = SyntaxNodeWithDb::new(&signature, db);
    let signature = quote! { #signature };

    let body = func.body(db).as_syntax_node();
    let body = SyntaxNodeWithDb::new(&body, db);

    let attributes = func.attributes(db).as_syntax_node();
    let attributes = SyntaxNodeWithDb::new(&attributes, db);

    let test_func = TokenStream::new(vec![format_ident!(
        "{}__snforge_internal_test_generated",
        name
    )]);
    let func_ident = format_ident!("{}", name);

    if should_run_test {
        let call_args = TokenStream::empty();

        let test_func_with_attrs = test_func_with_attrs(&test_func, &called_func, &call_args);

        Ok(quote!(
            #test_func_with_attrs

            #attributes
            #[#internal_config]
            fn #func_ident #signature
            #body
        ))
    } else {
        Ok(quote!(
            #[#internal_config]
            #func_item
        ))
    }
}

fn ensure_parameters_only_with_fuzzer_or_test_case_attribute(
    db: &SimpleParserDatabase,
    func: &FunctionWithBody,
) -> Result<(), Diagnostic> {
    if has_parameters(db, func)
        && !has_fuzzer_attribute(db, func)
        && !has_test_case_attribute(db, func)
    {
        Err(TestCollector::error(
            "function with parameters must have #[fuzzer] or #[test_case] attribute",
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

#[must_use]
pub fn test_func_with_attrs(
    test_fn_name: &TokenStream,
    fn_name: &TokenStream,
    call_args: &TokenStream,
) -> TokenStream {
    let test_fn_name = test_fn_name.clone();
    let fn_name = fn_name.clone();
    let call_args = call_args.clone();
    let out_of_gas = create_single_token("'Out of gas'");
    quote!(
        #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
        #[snforge_internal_test_executable]
        fn #test_fn_name(mut _data: Span<felt252>) -> Span::<felt252> {
            core::internal::require_implicit::<System>();
            core::internal::revoke_ap_tracking();
            core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), #out_of_gas);

            core::option::OptionTraitImpl::expect(
                core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), #out_of_gas
            );
            #fn_name (#call_args);

            let mut arr = ArrayTrait::new();
            core::array::ArrayTrait::span(@arr)
        }
    )
}
