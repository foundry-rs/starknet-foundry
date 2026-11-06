use crate::args::Arguments;
use crate::attributes::AttributeInfo;
use crate::attributes::finalize::{FinalizeCollector, config_fn_ident};
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
    // outer function, which therefore needs a config run of its own.
    let test_attr = attr_list.find_attr(db, TestCollector::ATTR_NAME);
    let test_runs_on_outer_fn = test_attr.is_some();
    let test_or_executable_attrs = test_attr.map_or_else(Vec::new, |attr| vec![attr]);

    // The marker is re-attached below, so drop any copy already present.
    let finalize_attrs = attr_list
        .query_attr(db, FinalizeCollector::ATTR_NAME)
        .collect::<Vec<_>>();
    let actual_body_fn_attrs = attr_list
        .elements(db)
        .filter(|attr| !finalize_attrs.contains(attr))
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

    let test_fn_name_text = func.declaration(db).name(db).text(db).to_string();
    let fuzzer_fn_name = format_ident!("{test_fn_name_text}__snforge_internal_fuzzer_generated");

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

    let test_fn_name = func.declaration(db).name(db).as_syntax_node();
    let test_fn_name = SyntaxNodeWithDb::new(&test_fn_name, db);

    let body = func.body(db).as_syntax_node();
    let body = SyntaxNodeWithDb::new(&body, db);

    let finalize_marker = create_single_token(FinalizeCollector::ATTR_NAME);
    let config_fn = config_fn_ident(&test_fn_name_text);

    // Config statements keep accumulating in the test function itself, and `#[__snforge_finalize]`
    // lifts them into `#config_fn`. When the test wrapper is generated for the outer function, its
    // config run just delegates there.
    let outer_config_run = if test_runs_on_outer_fn {
        quote!(
            if snforge_std::_internals::is_config_run() {
                #config_fn ();

                return;
            };
        )
    } else {
        TokenStream::empty()
    };

    Ok(quote!(
            #test_or_executable_attrs
            #vis fn #fuzzer_fn_name() {
                #outer_config_run
                #fuzzer_assignments
                let _ = #test_fn_name(#arguments_list);
            }

            #actual_body_fn_attrs
            #[#finalize_marker]
            fn #test_fn_name #signature
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
