use crate::{
    args::Arguments,
    attributes::{AttributeCollector, AttributeInfo, AttributeTypeData},
    cairo_expression::CairoExpression,
    config_statement::extend_with_config_cheatcodes,
    types::{Number, ParseFromExpr},
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::db::SyntaxGroup;

pub struct AvailableGasCollector;

impl AttributeInfo for AvailableGasCollector {
    const ATTR_NAME: &'static str = "available_gas";
}

impl AttributeTypeData for AvailableGasCollector {
    const CHEATCODE_NAME: &'static str = "set_config_available_gas";
}

impl AttributeCollector for AvailableGasCollector {
    fn args_into_config_expression(
        db: &dyn SyntaxGroup,
        args: Arguments,
        _warns: &mut Vec<Diagnostic>,
    ) -> Result<String, Diagnostics> {
        let &[arg] = args.unnamed_only::<Self>()?.of_length::<1, Self>()?;

        let gas = Number::parse_from_expr::<Self>(db, arg.1, arg.0.to_string().as_str())?;

        let gas = gas.as_cairo_expression();

        Ok(format!(
            "snforge_std::_config_types::AvailableGasConfig {{ gas: {gas} }}"
        ))
    }
}

#[must_use]
pub fn available_gas(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<AvailableGasCollector>(args, item)
}
