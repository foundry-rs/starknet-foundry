use super::AttributeInfo;
use crate::{
    args::Arguments,
    asserts::assert_is_used_once,
    common::{into_proc_macro_result, with_parsed_values},
    config_statement::append_config_statements,
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::FunctionWithBody;

pub struct InternalConfigStatementCollector;

impl AttributeInfo for InternalConfigStatementCollector {
    const ATTR_NAME: &'static str = "__internal_config_statement";
}

#[must_use]
pub fn internal_config_statement(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, |args, item, warns| {
        with_parsed_values::<InternalConfigStatementCollector>(
            args,
            item,
            warns,
            internal_config_statement_internal,
        )
    })
}

// we need to insert empty config statement in case there was no config used
// so function will be stopped in configuration mode run
#[expect(clippy::ptr_arg)]
#[expect(clippy::needless_pass_by_value)]
fn internal_config_statement_internal(
    db: &SimpleParserDatabase,
    func: &FunctionWithBody,
    _args_db: &SimpleParserDatabase,
    args: Arguments,
    _warns: &mut Vec<Diagnostic>,
) -> Result<TokenStream, Diagnostics> {
    assert_is_used_once::<InternalConfigStatementCollector>(db, func)?;
    args.assert_is_empty::<InternalConfigStatementCollector>()?;

    Ok(append_config_statements(db, func, TokenStream::empty()))
}
