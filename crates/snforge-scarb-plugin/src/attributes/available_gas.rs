use crate::{
    args::Arguments,
    attributes::{AttributeCollector, AttributeInfo, AttributeTypeData},
    branch,
    cairo_expression::CairoExpression,
    config_statement::extend_with_config_cheatcodes,
    types::{Number, ParseFromExpr},
};
use cairo_lang_macro::{quote, Diagnostic, Diagnostics, ProcMacroResult, Severity, TokenStream};
use cairo_lang_parser::utils::SimpleParserDatabase;

pub struct AvailableGasCollector;

impl AttributeInfo for AvailableGasCollector {
    const ATTR_NAME: &'static str = "available_gas";
}

impl AttributeTypeData for AvailableGasCollector {
    const CHEATCODE_NAME: &'static str = "set_config_available_gas";
}

impl AttributeCollector for AvailableGasCollector {
    fn args_into_config_expression(
        db: &SimpleParserDatabase,
        args: Arguments,
        _warns: &mut Vec<Diagnostic>,
    ) -> Result<TokenStream, Diagnostics> {
        let expr = branch!(from_resource_bounds(db, &args), from_max_gas(db, &args))?;
        Ok(expr)
    }
}

fn from_resource_bounds(
    db: &SimpleParserDatabase,
    args: &Arguments,
) -> Result<TokenStream, Diagnostic> {
    let named_args = args.named_only::<AvailableGasCollector>()?;
    let max = u64::MAX;
    let l1_gas = named_args
        .as_once_optional("l1_gas")?
        .map(|arg| Number::parse_from_expr::<AvailableGasCollector>(db, arg, "l1_gas"))
        .transpose()?
        .unwrap_or(Number(max.into()));

    let l1_data_gas = named_args
        .as_once_optional("l1_data_gas")?
        .map(|arg| Number::parse_from_expr::<AvailableGasCollector>(db, arg, "l1_data_gas"))
        .transpose()?
        .unwrap_or(Number(max.into()));

    let l2_gas = named_args
        .as_once_optional("l2_gas")?
        .map(|arg| Number::parse_from_expr::<AvailableGasCollector>(db, arg, "l2_gas"))
        .transpose()?
        .unwrap_or(Number(max.into()));

    l1_gas.validate_in_gas_range::<AvailableGasCollector>("l1_gas")?;
    l1_data_gas.validate_in_gas_range::<AvailableGasCollector>("l1_data_gas")?;
    l2_gas.validate_in_gas_range::<AvailableGasCollector>("l2_gas")?;

    let l1_gas_expr = l1_gas.as_cairo_expression();
    let l1_data_gas_expr = l1_data_gas.as_cairo_expression();
    let l2_gas_expr = l2_gas.as_cairo_expression();

    Ok(quote!(
        snforge_std::_internals::config_types::AvailableGasConfig::MaxResourceBounds(
             snforge_std::_internals::config_types::AvailableResourceBoundsConfig {
                 l1_gas: #l1_gas_expr,
                 l1_data_gas: #l1_data_gas_expr,
                 l2_gas: #l2_gas_expr,
             }
         )
    ))
}

fn from_max_gas(db: &SimpleParserDatabase, args: &Arguments) -> Result<TokenStream, Diagnostic> {
    let &[arg] = args
        .unnamed_only::<AvailableGasCollector>()?
        .of_length::<1, AvailableGasCollector>()?;

    let gas =
        Number::parse_from_expr::<AvailableGasCollector>(db, arg.1, arg.0.to_string().as_str())?;

    gas.validate_in_gas_range::<AvailableGasCollector>("max_gas")?;

    let gas = gas.as_cairo_expression();

    Ok(quote!(snforge_std::_internals::config_types::AvailableGasConfig::MaxGas(#gas)))
}

#[must_use]
pub fn available_gas(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<AvailableGasCollector>(args, item)
}
