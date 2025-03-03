use crate::{
    args::Arguments,
    attributes::{AttributeCollector, AttributeInfo, AttributeTypeData},
    cairo_expression::CairoExpression,
    config_statement::extend_with_config_cheatcodes,
    types::{Number, ParseFromExpr},
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::db::SyntaxGroup;
use num_bigint::BigInt;

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
        let named_args = args.named_only::<Self>()?;

        let l1_gas = named_args
            .as_once_optional("l1_gas")?
            .map(|arg| Number::parse_from_expr::<Self>(db, arg, "l1_gas"))
            .transpose()?
            .unwrap_or(Number(BigInt::ZERO));

        let l1_data_gas = named_args
            .as_once_optional("l1_data_gas")?
            .map(|arg| Number::parse_from_expr::<Self>(db, arg, "l1_data_gas"))
            .transpose()?
            .unwrap_or(Number(BigInt::ZERO));

        let l2_gas = named_args
            .as_once_optional("l2_gas")?
            .map(|arg| Number::parse_from_expr::<Self>(db, arg, "l2_gas"))
            .transpose()?
            .unwrap_or(Number(BigInt::ZERO));

        let l1_gas_expr = l1_gas.as_cairo_expression();
        let l1_data_gas_expr = l1_data_gas.as_cairo_expression();
        let l2_gas_expr = l2_gas.as_cairo_expression();

        Ok(format!(
            "snforge_std::_config_types::AvailableGasConfig {{ l1_gas: {l1_gas_expr}, l1_data_gas: {l1_data_gas_expr}, l2_gas: {l2_gas_expr} }}"
        ))
    }
}

#[must_use]
pub fn available_gas(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<AvailableGasCollector>(args, item)
}
