use super::{AttributeInfo, AttributeTypeData};
use crate::{
    args::Arguments, attributes::AttributeCollector,
    config_statement::extend_with_config_cheatcodes,
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::db::SyntaxGroup;

pub struct IgnoreCollector;

impl AttributeInfo for IgnoreCollector {
    const ATTR_NAME: &'static str = "ignore";
}

impl AttributeTypeData for IgnoreCollector {
    const CHEATCODE_NAME: &'static str = "set_config_ignore";
}

impl AttributeCollector for IgnoreCollector {
    fn args_into_config_expression(
        _db: &dyn SyntaxGroup,
        args: Arguments,
        _warns: &mut Vec<Diagnostic>,
    ) -> Result<String, Diagnostics> {
        args.assert_is_empty::<Self>()?;

        Ok(
            "snforge_std_deprecated::_internals::config_types::IgnoreConfig { is_ignored: true }"
                .to_string(),
        )
    }
}

#[must_use]
pub fn ignore(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<IgnoreCollector>(args, item)
}
