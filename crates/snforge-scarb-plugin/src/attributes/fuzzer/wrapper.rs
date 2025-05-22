use crate::args::Arguments;
use crate::attributes::internal_config_statement::InternalConfigStatementCollector;
use crate::attributes::test::TestCollector;
use crate::attributes::AttributeInfo;
use crate::common::{into_proc_macro_result, with_parsed_values};
use crate::utils::{create_single_token, get_statements, SyntaxNodeUtils};
use cairo_lang_macro::{quote, Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{FunctionWithBody, Param};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::TypedSyntaxNode;

pub struct FuzzerWrapperCollector;

impl AttributeInfo for FuzzerWrapperCollector {
    const ATTR_NAME: &'static str = "__fuzzer_wrapper";
}

#[must_use]
pub fn fuzzer_wrapper(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, |args, item, warns| {
        with_parsed_values::<FuzzerWrapperCollector>(args, item, warns, fuzzer_wrapper_internal)
    })
}

#[expect(clippy::ptr_arg)]
#[expect(clippy::needless_pass_by_value)]
fn fuzzer_wrapper_internal(
    db: &SimpleParserDatabase,
    func: &FunctionWithBody,
    _args_db: &SimpleParserDatabase,
    args: Arguments,
    _warns: &mut Vec<Diagnostic>,
) -> Result<TokenStream, Diagnostics> {
    args.assert_is_empty::<FuzzerWrapperCollector>()?;

    let attr_list = func.attributes(db);
    let test_or_executable_attrs =
        if let Some(test_attr) = attr_list.find_attr(db, TestCollector::ATTR_NAME) {
            vec![test_attr]
        } else {
            [
                attr_list.query_attr(db, "snforge_internal_test_executable"),
                attr_list.query_attr(db, InternalConfigStatementCollector::ATTR_NAME),
            ]
            .concat()
        };

    let actual_body_fn_attrs = attr_list
        .elements(db)
        .into_iter()
        .filter(|attr| !test_or_executable_attrs.contains(attr))
        .map(|stmt| stmt.to_token_stream(db))
        .fold(TokenStream::empty(), |mut acc, token| {
            acc.extend(token);
            acc
        });

    let test_or_executable_attrs = test_or_executable_attrs
        .iter()
        .map(|stmt| stmt.to_token_stream(db))
        .fold(TokenStream::empty(), |mut acc, token| {
            acc.extend(token);
            acc
        });

    let vis = func.visibility(db).as_syntax_node();
    let vis = SyntaxNodeWithDb::new(&vis, db);

    let name = func.declaration(db).name(db).as_syntax_node();
    let name = SyntaxNodeWithDb::new(&name, db);

    let signature = func.declaration(db).signature(db).as_syntax_node();
    let signature = SyntaxNodeWithDb::new(&signature, db);

    let fuzzer_assignments = extract_and_transform_params(db, func, |param| {
        let name = param.name(db).as_syntax_node();
        let name = SyntaxNodeWithDb::new(&name, db);

        let name_type = param.type_clause(db).as_syntax_node();
        let name_type = SyntaxNodeWithDb::new(&name_type, db);

        quote! {
            let #name #name_type = snforge_std::fuzzable::Fuzzable::generate();
            snforge_std::_internals::save_fuzzer_arg(@#name);
        }
    });

    let blank_values_for_config_run = extract_and_transform_params(db, func, |_param| {
        quote! {
            snforge_std::fuzzable::Fuzzable::blank(),
        }
    });

    let arguments_list = extract_and_transform_params(db, func, |param| {
        let name = param.name(db).as_syntax_node();
        let name = SyntaxNodeWithDb::new(&name, db);

        quote! {
            #name,
        }
    });

    let actual_body_fn_name = TokenStream::new(vec![create_single_token(format!(
        "{}_actual_body",
        func.declaration(db).name(db).as_text(db)
    ))]);

    let (statements, if_content) = get_statements(db, func);

    let internal_config_attr = create_single_token(InternalConfigStatementCollector::ATTR_NAME);

    Ok(quote!(
            #test_or_executable_attrs
            #vis fn #name() {
                if snforge_std::_internals::is_config_run() {
                    #if_content

                    #actual_body_fn_name(#blank_values_for_config_run);

                    return;
                }
                #fuzzer_assignments
                #actual_body_fn_name(#arguments_list);
            }

            #actual_body_fn_attrs
            #[#internal_config_attr]
            fn #actual_body_fn_name #signature {
                #statements
            }
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
        .iter()
        .map(transformer)
        .fold(TokenStream::empty(), |mut acc, token| {
            acc.extend(token);
            acc
        })
}
