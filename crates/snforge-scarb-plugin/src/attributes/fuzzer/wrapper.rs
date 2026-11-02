use crate::args::Arguments;
use crate::attributes::AttributeInfo;
use crate::attributes::finalize::{FinalizeCollector, config_fn_name};
use crate::attributes::internal_config_statement::InternalConfigStatementCollector;
use crate::attributes::test::TestCollector;
use crate::common::{into_proc_macro_result, with_parsed_values};
use crate::format_ident;
use crate::utils::{SyntaxNodeUtils, create_single_token};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::OptionTypeClause::{Empty, TypeClause};
use cairo_lang_syntax::node::ast::{FunctionWithBody, Param};
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode};

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
    // `#[test]` still being here means it has not expanded yet and will run on the generated
    // outer function. The config statements have to travel with it, so that the finalize marker
    // it attaches ends up on the same function that holds them.
    let test_runs_on_outer_fn = attr_list.find_attr(db, TestCollector::ATTR_NAME).is_some();
    let test_or_executable_attrs =
        if let Some(test_attr) = attr_list.find_attr(db, TestCollector::ATTR_NAME) {
            vec![test_attr]
        } else {
            attr_list
                .query_attr(db, InternalConfigStatementCollector::ATTR_NAME)
                .collect::<Vec<_>>()
        };

    let finalize_attr_text = format!("#[{}]", FinalizeCollector::ATTR_NAME);
    let actual_body_fn_attrs = attr_list
        .elements(db)
        .filter(|attr| attr.as_syntax_node().get_text(db).trim() != finalize_attr_text)
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

    let name = format_ident!(
        "{}__snforge_internal_fuzzer_generated",
        func.declaration(db).name(db).text(db)
    );

    let signature = func.declaration(db).signature(db).as_syntax_node();
    let signature = SyntaxNodeWithDb::new(&signature, db);

    let fuzzer_assignments = extract_and_transform_params(db, func, |param| {
        let name = param.name(db).as_syntax_node();
        let name = SyntaxNodeWithDb::new(&name, db);

        let name_type = match param.type_clause(db) {
            TypeClause(type_clause) => type_clause.ty(db).as_syntax_node(),
            Empty(option_type_clause) => option_type_clause.as_syntax_node(),
        };
        let name_type = SyntaxNodeWithDb::new(&name_type, db);

        quote! {
            let #name = snforge_std::fuzzable::Fuzzable::<#name_type>::generate();
            snforge_std::_internals::save_fuzzer_arg(@#name);
        }
    });

    let arguments_list = extract_and_transform_params(db, func, |param| {
        let name = param.name(db).as_syntax_node();
        let name = SyntaxNodeWithDb::new(&name, db);

        quote! {
            #name,
        }
    });

    let func_name = func.declaration(db).name(db).as_syntax_node();
    let func_name = SyntaxNodeWithDb::new(&func_name, db);

    let body = func.body(db).as_syntax_node();
    let body = SyntaxNodeWithDb::new(&body, db);

    let finalize_marker = create_single_token(FinalizeCollector::ATTR_NAME);
    let inner_config_fn = config_fn_name(&func.declaration(db).name(db).text(db));

    // Config statements keep accumulating in the test function itself, and `#[__snforge_finalize]`
    // lifts them into `#inner_config_fn`. When `#[test]` has not expanded yet it will generate the
    // test wrapper for the outer function, so that one needs a config run of its own - it just
    // delegates to the test function's config.
    let outer_config_run = if test_runs_on_outer_fn {
        quote!(
            if snforge_std::_internals::is_config_run() {
                #inner_config_fn ();

                return Default::default();
            };
        )
    } else {
        TokenStream::empty()
    };

    Ok(quote!(
            #test_or_executable_attrs
            #vis fn #name() {
                #outer_config_run
                #fuzzer_assignments
                let _ = #func_name(#arguments_list);
            }

            #actual_body_fn_attrs
            #[#finalize_marker]
            fn #func_name #signature
            #body
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
