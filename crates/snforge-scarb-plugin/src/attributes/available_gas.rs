use crate::{
    args::{Arguments, unnamed::UnnamedArgs},
    attributes::{AttributeCollector, AttributeInfo, AttributeTypeData, ErrorExt},
    cairo_expression::CairoExpression,
    config_statement::extend_with_config_cheatcodes,
    types::{Number, ParseFromExpr},
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::Expr;

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
        match args.unnamed_only::<AvailableGasCollector>() {
            // Empty argument list is handled by `from_named_args`, so that its
            // "empty argument list" warning is emitted.
            Ok(unnamed_args) if !args.is_empty() => Ok(from_unnamed_sierra_gas(db, &unnamed_args)?),
            _ => Ok(from_named_args(db, &args)?),
        }
    }
}

// A single unnamed argument sets the Sierra gas limit, e.g. `#[available_gas(100)]`.
fn from_unnamed_sierra_gas(
    db: &SimpleParserDatabase,
    args: &UnnamedArgs,
) -> Result<TokenStream, Diagnostic> {
    let &[arg] = args.of_length::<1, AvailableGasCollector>()?;

    let gas = parse_gas::<AvailableGasCollector>(db, arg.1, "gas")?;

    Ok(max_sierra_gas_config(&gas))
}

fn from_named_args(db: &SimpleParserDatabase, args: &Arguments) -> Result<TokenStream, Diagnostic> {
    let named_args = args.named_only::<AvailableGasCollector>(
        db,
        &["l1_gas", "l1_data_gas", "l2_gas", "sierra_gas"],
    )?;

    let l1_gas = named_args.as_once_optional("l1_gas")?;
    let l1_data_gas = named_args.as_once_optional("l1_data_gas")?;
    let l2_gas = named_args.as_once_optional("l2_gas")?;
    let sierra_gas = named_args.as_once_optional("sierra_gas")?;

    // `sierra_gas` uses the unified Sierra gas model, while `l1_gas`, `l1_data_gas` and `l2_gas`
    // use the separate resource-bounds model. The two models are mutually exclusive, so setting
    // `sierra_gas` together with any resource bound is rejected.
    if let Some(sierra_gas) = sierra_gas {
        if l1_gas.is_some() || l1_data_gas.is_some() || l2_gas.is_some() {
            return Err(AvailableGasCollector::error(
                "`sierra_gas` cannot be combined with resource bounds `l1_gas`, `l1_data_gas`, `l2_gas`",
            ));
        }

        let gas = parse_gas::<AvailableGasCollector>(db, sierra_gas, "sierra_gas")?;

        return Ok(max_sierra_gas_config(&gas));
    }

    from_resource_bounds(db, l1_gas, l1_data_gas, l2_gas)
}

fn from_resource_bounds(
    db: &SimpleParserDatabase,
    l1_gas: Option<&Expr>,
    l1_data_gas: Option<&Expr>,
    l2_gas: Option<&Expr>,
) -> Result<TokenStream, Diagnostic> {
    // Unset resource bounds default to `u64::MAX`, i.e. no limit.
    let max = u64::MAX;
    let l1_gas = l1_gas
        .map(|arg| parse_gas::<AvailableGasCollector>(db, arg, "l1_gas"))
        .transpose()?
        .unwrap_or(Number(max.into()));

    let l1_data_gas = l1_data_gas
        .map(|arg| parse_gas::<AvailableGasCollector>(db, arg, "l1_data_gas"))
        .transpose()?
        .unwrap_or(Number(max.into()));

    let l2_gas = l2_gas
        .map(|arg| parse_gas::<AvailableGasCollector>(db, arg, "l2_gas"))
        .transpose()?
        .unwrap_or(Number(max.into()));

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

// Parses a gas value and validates that it fits in the supported gas range.
fn parse_gas<T: AttributeInfo>(
    db: &SimpleParserDatabase,
    expr: &Expr,
    arg_name: &str,
) -> Result<Number, Diagnostic> {
    let gas = Number::parse_from_expr::<T>(db, expr, arg_name)?;
    gas.validate_in_gas_range::<T>(arg_name)?;

    Ok(gas)
}

fn max_sierra_gas_config(gas: &Number) -> TokenStream {
    let gas_expr = gas.as_cairo_expression();

    quote!(
        snforge_std::_internals::config_types::AvailableGasConfig::MaxSierraGas(#gas_expr)
    )
}

#[must_use]
pub fn available_gas(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<AvailableGasCollector>(args, item)
}
