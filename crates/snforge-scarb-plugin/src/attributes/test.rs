use super::{internal_config_statement::InternalConfigStatementCollector, AttributeInfo, ErrorExt};
use crate::attributes::fuzzer::wrapper::FuzzerWrapperCollector;
use crate::attributes::fuzzer::{FuzzerCollector, FuzzerConfigCollector};
use crate::utils::create_single_token;
use crate::{
    args::Arguments,
    common::{into_proc_macro_result, with_parsed_values},
};
use cairo_lang_macro::{quote, Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::{ast::FunctionWithBody, Terminal, TypedSyntaxNode};
use std::env::{self, VarError};
use std::ops::Not;

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

    let internal_config = create_single_token(InternalConfigStatementCollector::ATTR_NAME);

    let func_item = func.as_syntax_node();
    let func_item = SyntaxNodeWithDb::new(&func_item, db);

    let name = func.declaration(db).name(db).text(db).to_string();

    let test_filter = get_forge_test_filter().ok();

    let should_run_test = match test_filter {
        Some(ref filter) => name.contains(filter),
        None => true,
    };

    let body = func.body(db).as_syntax_node();
    let body = SyntaxNodeWithDb::new(&body, db);

    let attrs = func.attributes(db).as_syntax_node();
    let attrs = SyntaxNodeWithDb::new(&attrs, db);

    let vis = func.visibility(db).as_syntax_node();
    let vis = SyntaxNodeWithDb::new(&vis, db);

    let declaration = func.declaration(db).as_syntax_node();
    let declaration = SyntaxNodeWithDb::new(&declaration, db);

    if should_run_test {
        Ok(quote!(
            #[#internal_config]
            #attrs
            #[snforge_internal_test_executable]
            #vis #declaration
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
        .is_empty()
        .not()
}

fn no_fuzzer_attribute(db: &SimpleParserDatabase, func: &FunctionWithBody) -> bool {
    const FUZZER_ATTRIBUTES: [&str; 3] = [
        FuzzerCollector::ATTR_NAME,
        FuzzerWrapperCollector::ATTR_NAME,
        FuzzerConfigCollector::ATTR_NAME,
    ];

    func.attributes(db)
        .elements(db)
        .iter()
        .any(|attr| {
            FUZZER_ATTRIBUTES.contains(
                &attr
                    .attr(db)
                    .as_syntax_node()
                    .get_text_without_trivia(db)
                    .as_str(),
            )
        })
        .not()
}
