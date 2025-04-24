use super::{AttributeInfo, AttributeTypeData};
use crate::{
    args::Arguments, attributes::AttributeCollector,
    config_statement::extend_with_config_cheatcodes,
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::db::SyntaxGroup;

pub struct StrkPredeploymentCollector;

impl AttributeInfo for StrkPredeploymentCollector {
    const ATTR_NAME: &'static str = "disable_strk_predeployment";
}

impl AttributeTypeData for StrkPredeploymentCollector {
    const CHEATCODE_NAME: &'static str = "set_config_strk_predeployment";
}

impl AttributeCollector for StrkPredeploymentCollector {
    fn args_into_config_expression(
        _db: &dyn SyntaxGroup,
        args: Arguments,
        _warns: &mut Vec<Diagnostic>,
    ) -> Result<String, Diagnostics> {
        args.assert_is_empty::<Self>()?;

        Ok("snforge_std::_config_types::StrkPredeploymentConfig { is_disabled: true }".to_string())
    }
}

#[must_use]
pub fn disable_strk_predeployment(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<StrkPredeploymentCollector>(args, item)
}
