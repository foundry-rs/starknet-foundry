use super::{AttributeInfo, AttributeTypeData};
use crate::{
    args::Arguments, attributes::AttributeCollector, config_fn::ExtendWithConfig, MacroResult,
};
use cairo_lang_macro::{Diagnostics, TokenStream};
use cairo_lang_syntax::node::db::SyntaxGroup;

pub struct FuzzerCollector;

impl AttributeInfo for FuzzerCollector {
    const ATTR_NAME: &'static str = "fuzzer";
    const ARGS_FORM: &'static str = "<runs>: `u64`, <seed>: `felt252`";
}

impl AttributeTypeData for FuzzerCollector {
    const CHEATCODE_NAME: &'static str = "set_config_fuzzer";
}

impl AttributeCollector for FuzzerCollector {
    fn args_into_body(db: &dyn SyntaxGroup, args: Arguments) -> Result<String, Diagnostics> {
        let named_args = args.named_only::<Self>()?;

        let seed = validate::maybe_number_value::<Self>(db, named_args, "seed")?;
        let runs = validate::maybe_number_value::<Self>(db, named_args, "runs")?;

        Ok(format!(
            "snforge_std::_config_types::FuzzerConfig {{ seed: {seed}, runs: {runs} }}"
        ))
    }
}

pub fn _fuzzer(args: TokenStream, item: TokenStream) -> MacroResult {
    FuzzerCollector::extend_with_config_cheatcodes(args, item)
}

pub mod validate {
    use crate::{
        args::named::NamedArgs,
        attributes::{AttributeInfo, ErrorExt},
    };
    use cairo_lang_macro::Diagnostic;
    use cairo_lang_syntax::node::{ast::Expr, db::SyntaxGroup};

    pub fn maybe_number_value<T: AttributeInfo>(
        db: &dyn SyntaxGroup,
        named_args: &NamedArgs,
        arg_name: &str,
    ) -> Result<String, Diagnostic> {
        let arg = named_args.as_once_optional(arg_name)?;

        arg.map_or(Ok("Option::None".to_string()), |arg| {
            number::<T>(db, arg, arg_name).map(|num| format!("Option::Some({num})"))
        })
    }

    pub fn number<T: AttributeInfo>(
        db: &dyn SyntaxGroup,
        num: &Expr,
        arg: &str,
    ) -> Result<String, Diagnostic> {
        match num {
            Expr::Literal(literal) => {
                let num = literal
                    .numeric_value(db)
                    .ok_or_else(|| T::error(format!("<{arg}> got invalid number literal")))?
                    .to_str_radix(16);

                Ok(format!("0x{num}"))
            }
            _ => Err(T::error(format!("<{arg}> should be number literal",))),
        }
    }
}
