use super::{AttributeInfo, AttributeTypeData};
use crate::{
    args::Arguments, attributes::AttributeCollector,
    config_statement::extend_with_config_cheatcodes,
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;

pub struct PredeployedContractsCollector;

impl AttributeInfo for PredeployedContractsCollector {
    const ATTR_NAME: &'static str = "disable_predeployed_contracts";
}

impl AttributeTypeData for PredeployedContractsCollector {
    const CHEATCODE_NAME: &'static str = "set_config_disable_contracts";
}

impl AttributeCollector for PredeployedContractsCollector {
    fn args_into_config_expression(
        _db: &SimpleParserDatabase,
        args: Arguments,
        _warns: &mut Vec<Diagnostic>,
    ) -> Result<TokenStream, Diagnostics> {
        args.assert_is_empty::<Self>()?;

        Ok(quote! {
            snforge_std::_internals::config_types::PredeployedContractsConfig { is_disabled: true }
        })
    }
}

#[must_use]
pub fn disable_predeployed_contracts(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<PredeployedContractsCollector>(args, item)
}
