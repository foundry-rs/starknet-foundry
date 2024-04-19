use super::{AttributeInfo, AttributeReturnType};
use crate::{
    args::Arguments,
    attributes::{AttributeCollector, ErrorExt},
    config_fn::ConfigFn,
    MacroResult,
};
use cairo_lang_macro::{Diagnostics, TokenStream};
use cairo_lang_syntax::node::{ast::Expr, db::SyntaxGroup, Terminal};

pub struct AvailableGasCollector;

impl AttributeInfo for AvailableGasCollector {
    const ATTR_NAME: &'static str = "available_gas";
    const ARGS_FORM: &'static str = "<usize>";
}

impl AttributeReturnType for AvailableGasCollector {
    const RETURN_TYPE: &'static str = "AvailableGasConfig";
    const EXECUTABLE_NAME: &'static str = "__snforge_available_gas__";
}

impl AttributeCollector for AvailableGasCollector {
    fn args_into_body(db: &dyn SyntaxGroup, args: Arguments) -> Result<String, Diagnostics> {
        let [arg] = args.unnamed_only::<Self>()?.of_length::<1>()?;

        let gas: u64 = match arg {
            Expr::Literal(literal) => match literal.text(db).parse() {
                Ok(gas) => gas,
                Err(_) => Err(Self::error("argument should be number"))?,
            },
            _ => Err(Self::error("argument should be number"))?,
        };

        Ok(format!("gas: {gas}"))
    }
}

pub fn _available_gas(args: TokenStream, item: TokenStream) -> MacroResult {
    AvailableGasCollector::extend_with_config_fn(args, item)
}
