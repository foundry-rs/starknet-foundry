use super::{AttributeInfo, AttributeTypeData};
use crate::{
    args::Arguments, attributes::AttributeCollector,
    config_statement::extend_with_config_cheatcodes,
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::db::SyntaxGroup;

pub struct ContractsPredeploymentCollector;

impl AttributeInfo for ContractsPredeploymentCollector {
    const ATTR_NAME: &'static str = "disable_predeployed_contracts";
}

impl AttributeTypeData for ContractsPredeploymentCollector {
    const CHEATCODE_NAME: &'static str = "set_config_contracts_predeployment";
}

impl AttributeCollector for ContractsPredeploymentCollector {
    fn args_into_config_expression(
        _db: &dyn SyntaxGroup,
        args: Arguments,
        _warns: &mut Vec<Diagnostic>,
    ) -> Result<String, Diagnostics> {
        args.assert_is_empty::<Self>()?;

        Ok(
            "snforge_std::_internals::config_types::ContractsPredeploymentConfig { is_disabled: true }"
                .to_string(),
        )
    }
}

#[must_use]
pub fn disable_predeployed_contracts(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<ContractsPredeploymentCollector>(args, item)
}
