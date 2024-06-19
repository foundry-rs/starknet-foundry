use super::{AttributeInfo, AttributeTypeData};
use crate::{
    args::Arguments,
    attributes::AttributeCollector,
    cairo_expression::CairoExpression,
    config_statement::extend_with_config_cheatcodes,
    types::{Number, ParseFromExpr},
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::db::SyntaxGroup;

pub struct FuzzerCollector;

impl AttributeInfo for FuzzerCollector {
    const ATTR_NAME: &'static str = "fuzzer";
    const ARGS_FORM: &'static str = "<runs>: number (greater than 0), <seed>: number";
}

impl AttributeTypeData for FuzzerCollector {
    const CHEATCODE_NAME: &'static str = "set_config_fuzzer";
}

impl AttributeCollector for FuzzerCollector {
    fn args_into_config_expression(
        db: &dyn SyntaxGroup,
        args: Arguments,
        _warns: &mut Vec<Diagnostic>,
    ) -> Result<String, Diagnostics> {
        let named_args = args.named_only::<Self>()?;

        let seed = named_args
            .as_once_optional("seed")?
            .map(|arg| Number::parse_from_expr::<Self>(db, arg, "seed"))
            .transpose()?;

        let runs = named_args
            .as_once_optional("runs")?
            .map(|arg| Number::parse_from_expr::<Self>(db, arg, "runs"))
            .transpose()?;

        //TODO validate runs > 0

        let seed = seed.as_cairo_expression();
        let runs = runs.as_cairo_expression();

        Ok(format!(
            "snforge_std::_config_types::FuzzerConfig {{ seed: {seed}, runs: {runs} }}"
        ))
    }
}

#[must_use]
pub fn fuzzer(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<FuzzerCollector>(args, item)
}