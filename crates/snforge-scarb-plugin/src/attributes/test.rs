use super::AttributeInfo;
use crate::{config_statement::append_config_statements, parse::parse};
use cairo_lang_macro::{ProcMacroResult, TokenStream};
use cairo_lang_utils::Upcast;

struct TestCollector;

impl AttributeInfo for TestCollector {
    const ATTR_NAME: &'static str = "test";
    const ARGS_FORM: &'static str = "";
}

pub fn test(item: TokenStream) -> ProcMacroResult {
    match parse::<TestCollector>(&item.to_string()) {
        Ok((db, func)) => ProcMacroResult::new(TokenStream::new(
            // we need to insert empty config statement in case there was no config used
            append_config_statements(db.upcast(), &func, ""),
        )),
        Err(diagnostics) => ProcMacroResult::new(item).with_diagnostics(diagnostics.into()),
    }
}
