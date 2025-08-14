use crate::args::Arguments;
use crate::attributes::internal_config_statement::InternalConfigStatementCollector;
use crate::attributes::test_case::TestCaseCollector;
use crate::attributes::{AttributeInfo, ErrorExt};
use crate::common::{into_proc_macro_result, with_parsed_values};
use crate::format_ident;
use crate::utils::{SyntaxNodeUtils, create_single_token, get_statements};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{FunctionWithBody, OptionArgListParenthesized, Param};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode};

pub struct ParamTestWrapperCollector;
impl AttributeInfo for ParamTestWrapperCollector {
    const ATTR_NAME: &'static str = "__param_test_wrapper";
}

#[must_use]
pub fn param_test_wrapper(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, |args, item, warns| {
        with_parsed_values::<ParamTestWrapperCollector>(
            args,
            item,
            warns,
            param_test_wrapper_internal,
        )
    })
}

#[expect(clippy::ptr_arg)]
#[expect(clippy::needless_pass_by_value)]
fn param_test_wrapper_internal(
    db: &SimpleParserDatabase,
    func: &FunctionWithBody,
    _args_db: &SimpleParserDatabase,
    args: Arguments,
    _warns: &mut Vec<Diagnostic>,
) -> Result<TokenStream, Diagnostics> {
    args.assert_is_empty::<ParamTestWrapperCollector>()?;

    let attr_list = func.attributes(db);

    let case_attrs: Vec<_> = attr_list
        .query_attr(db, TestCaseCollector::ATTR_NAME)
        .collect();
    if case_attrs.is_empty() {
        Err(ParamTestWrapperCollector::error(
            "No #[test_case(...)] found. Add at least one.",
        ))?;
    }

    let actual_body_fn_attrs = attr_list
        .elements(db)
        .filter(|attr| {
            let binding = attr.as_syntax_node().get_text(db);
            let name = binding.as_str();
            println!("name: {name}");
            !name.contains(TestCaseCollector::ATTR_NAME)
                && !name.contains(super::ParamTestCollector::ATTR_NAME)
        })
        .map(|stmt| stmt.to_token_stream(db))
        .fold(TokenStream::empty(), |mut acc, token| {
            acc.extend(token);
            acc
        });

    let vis = func.visibility(db).as_syntax_node();
    let vis = SyntaxNodeWithDb::new(&vis, db);

    let name_node = func.declaration(db).name(db);
    let name = SyntaxNodeWithDb::new(&name_node.as_syntax_node(), db);

    let signature = func.declaration(db).signature(db).as_syntax_node();
    let signature = SyntaxNodeWithDb::new(&signature, db);

    let actual_body_fn_name = format_ident!("{}_actual_body", name_node.text(db));

    let (statements, if_content) = get_statements(db, func);

    let internal_config_attr = create_single_token(InternalConfigStatementCollector::ATTR_NAME);

    let blank_values_for_config_run = extract_and_transform_params(db, func, |_param| {
        quote! { snforge_std::fuzzable::Fuzzable::blank(), }
    });

    let mut generated_tests = TokenStream::empty();
    for (_, case_attr) in case_attrs.iter().enumerate() {
        let case_args_str = if let OptionArgListParenthesized::ArgListParenthesized(args) =
            case_attr.arguments(db)
        {
            args.arguments(db)
                .elements(db)
                .map(|e| e.as_syntax_node().get_text(db))
                .collect::<Vec<_>>()
                .join("_")
        } else {
            String::new()
        };

        println!("case_args_str: {case_args_str}");

        let case_args_ts = case_attr.arguments(db).to_token_stream(db);

        let case_fn_ident = format!("{}_{}", name_node.text(db), case_args_str);
        let case_fn_name = format_ident!("{}", case_fn_ident);

        let test_attr = create_single_token("test");

        generated_tests.extend(quote! {
            #[#test_attr]
            #vis fn #case_fn_name() {
                if snforge_std::_internals::is_config_run() {
                    #if_content
                    #actual_body_fn_name(#blank_values_for_config_run);
                    return;
                }
                #actual_body_fn_name #case_args_ts;
            }
        });
    }

    Ok(quote!(
        #actual_body_fn_attrs
        #[#internal_config_attr]
        fn #actual_body_fn_name #signature {
            #statements
        }

        #generated_tests
    ))
}

fn extract_and_transform_params<F>(
    db: &SimpleParserDatabase,
    func: &FunctionWithBody,
    transformer: F,
) -> TokenStream
where
    F: Fn(&Param) -> TokenStream,
{
    func.declaration(db)
        .signature(db)
        .parameters(db)
        .elements(db)
        .map(|e| transformer(&e))
        .fold(TokenStream::empty(), |mut acc, token| {
            acc.extend(token);
            acc
        })
}
