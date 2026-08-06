use crate::{
    args::{Arguments, unnamed::UnnamedArgs},
    attributes::{AttributeCollector, AttributeInfo, AttributeTypeData},
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
            Ok(unnamed_args) if !args.is_empty() => Ok(from_unnamed_l2_gas(db, &unnamed_args)?),
            _ => Ok(from_named_args(db, &args)?),
        }
    }
}

// A single unnamed argument sets the L2 gas limit, e.g. `#[available_gas(100)]`.
fn from_unnamed_l2_gas(
    db: &SimpleParserDatabase,
    args: &UnnamedArgs,
) -> Result<TokenStream, Diagnostic> {
    let &[arg] = args.of_length::<1, AvailableGasCollector>()?;

    from_resource_bounds(db, None, None, Some(arg.1))
}

fn from_named_args(db: &SimpleParserDatabase, args: &Arguments) -> Result<TokenStream, Diagnostic> {
    let named_args = args.named_only::<AvailableGasCollector>(
        db,
        &["l1_gas", "l1_data_gas", "l2_gas"],
    )?;

    let l1_gas = named_args.as_once_optional("l1_gas")?;
    let l1_data_gas = named_args.as_once_optional("l1_data_gas")?;
    let l2_gas = named_args.as_once_optional("l2_gas")?;

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

#[must_use]
pub fn available_gas(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<AvailableGasCollector>(args, item)
}
