use super::AttributeInfo;
use crate::args::Arguments;
use crate::asserts::assert_is_used_once;
use crate::common::{into_proc_macro_result, with_parsed_values};
use crate::format_ident;
use crate::utils::get_statements;
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::FunctionWithBody;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode};

pub struct ExtractConfigCollector;

impl AttributeInfo for ExtractConfigCollector {
    const ATTR_NAME: &'static str = "__snforge_extract_config";
}

#[must_use]
pub fn config_fn_ident(test_name: &str) -> TokenStream {
    TokenStream::new(vec![format_ident!("{test_name}__snforge_internal_config")])
}

/// Config of the fuzzed test only: the shared config plus the fuzzer config. Explicit
/// `#[test_case]`s use [`config_fn_ident`], so they are not fuzzed.
#[must_use]
pub fn fuzzer_config_fn_ident(test_name: &str) -> TokenStream {
    TokenStream::new(vec![format_ident!(
        "{test_name}__snforge_internal_fuzzer_config"
    )])
}

#[must_use]
pub fn extract_config(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, |args, item, warns| {
        with_parsed_values::<ExtractConfigCollector>(args, item, warns, extract_config_internal)
    })
}

#[expect(clippy::needless_pass_by_value)]
fn extract_config_internal(
    db: &SimpleParserDatabase,
    func: &FunctionWithBody,
    _args_db: &SimpleParserDatabase,
    args: Arguments,
    _warns: &mut Vec<Diagnostic>,
) -> Result<TokenStream, Diagnostics> {
    assert_is_used_once::<ExtractConfigCollector>(db, func)?;
    args.assert_is_empty::<ExtractConfigCollector>()?;

    let name = func.declaration(db).name(db).text(db).to_string();
    let config_fn = config_fn_ident(&name);

    let (statements, if_content) = get_statements(db, func);

    let attributes = func.attributes(db).as_syntax_node();
    let attributes = SyntaxNodeWithDb::new(&attributes, db);
    let vis = func.visibility(db).as_syntax_node();
    let vis = SyntaxNodeWithDb::new(&vis, db);
    let declaration = func.declaration(db).as_syntax_node();
    let declaration = SyntaxNodeWithDb::new(&declaration, db);

    Ok(quote!(
        fn #config_fn() {
            #if_content
        }

        #attributes
        #vis #declaration {
            #statements
        }
    ))
}
