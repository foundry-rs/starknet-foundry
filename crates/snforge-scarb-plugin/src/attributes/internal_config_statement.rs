use super::AttributeInfo;
use crate::{
    asserts::assert_is_used_once, config_statement::append_config_statements, parse::parse,
};
use cairo_lang_macro::{ProcMacroResult, TokenStream};
use cairo_lang_utils::Upcast;

pub struct InternalConfigStatementCollector;

impl AttributeInfo for InternalConfigStatementCollector {
    const ATTR_NAME: &'static str = "__internal_config_statement";
    const ARGS_FORM: &'static str = "";
}

// we need to insert empty config statement in case there was no config used
// so function will be stopped in configuration mode run
pub fn internal_config_statement(item: TokenStream) -> ProcMacroResult {
    let parse_and_assert_result = parse::<InternalConfigStatementCollector>(&item.to_string())
        .and_then(|(db, func)| {
            assert_is_used_once::<InternalConfigStatementCollector>(db.upcast(), &func)?;

            Ok((db, func))
        });

    match parse_and_assert_result {
        Ok((db, func)) => ProcMacroResult::new(TokenStream::new(append_config_statements(
            db.upcast(),
            &func,
            "",
        ))),
        Err(diagnostics) => ProcMacroResult::new(item).with_diagnostics(diagnostics.into()),
    }
}
