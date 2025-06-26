use self::expected::Expected;
use crate::{
    args::Arguments,
    attributes::{AttributeCollector, AttributeInfo, AttributeTypeData},
    cairo_expression::CairoExpression,
    config_statement::extend_with_config_cheatcodes,
    types::ParseFromExpr,
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::db::SyntaxGroup;

mod expected;

pub struct ShouldPanicCollector;

impl AttributeInfo for ShouldPanicCollector {
    const ATTR_NAME: &'static str = "should_panic";
}

impl AttributeTypeData for ShouldPanicCollector {
    const CHEATCODE_NAME: &'static str = "set_config_should_panic";
}

impl AttributeCollector for ShouldPanicCollector {
    fn args_into_config_expression(
        db: &dyn SyntaxGroup,
        args: Arguments,
        _warns: &mut Vec<Diagnostic>,
    ) -> Result<String, Diagnostics> {
        let named_args = args.named_only::<Self>()?;

        let expected = named_args.as_once_optional("expected")?;

        let expected = expected
            .map(|expr| Expected::parse_from_expr::<Self>(db, expr, "expected"))
            .transpose()?
            .unwrap_or_default();

        let expected = expected.as_cairo_expression();

        Ok(format!(
            "snforge_std_compatibility::_internals::config_types::ShouldPanicConfig {{ expected: {expected} }}"
        ))
    }
}

#[must_use]
pub fn should_panic(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<ShouldPanicCollector>(args, item)
}
