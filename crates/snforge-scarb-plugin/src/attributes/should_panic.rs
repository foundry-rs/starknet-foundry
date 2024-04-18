use super::{AttributeInfo, AttributeReturnType};
use crate::{args::Arguments, attributes::AttributeCollector, config_fn::ConfigFn, MacroResult};
use cairo_lang_macro::{Diagnostics, TokenStream};
use cairo_lang_syntax::node::db::SyntaxGroup;
use std::fmt::Display;

pub struct ShouldPanicCollector;

impl AttributeInfo for ShouldPanicCollector {
    const ATTR_NAME: &'static str = "should_panic";
    const ARGS_FORM: &'static str =
        "[<expected>: `ByteArray` | `felt252` | ([`ByteArray` | `felt252`,])]";
}

impl AttributeReturnType for ShouldPanicCollector {
    const RETURN_TYPE: &'static str = "ShouldPanicConfig";
}

#[derive(Debug, Clone)]
enum CairoString {
    Short(String),
    Normal(String),
}

impl Display for CairoString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal(string) => write!(f, r#"String::Normal("{string}")"#),
            Self::Short(string) => write!(f, "String::Short('{string}')"),
        }
    }
}

impl AttributeCollector for ShouldPanicCollector {
    fn args_into_body(db: &dyn SyntaxGroup, args: Arguments) -> Result<String, Diagnostics> {
        let named_args = args.named_only::<Self>()?;

        let expected = named_args.as_once_optional("expected")?;

        let expected = expected
            .map(|expr| validate::list_of_strings::<Self>(db, expr))
            .transpose()?
            .unwrap_or_default();

        let expected = expected
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");

        Ok(format!("expected: array![{expected}]"))
    }
}

pub fn _should_panic(args: TokenStream, item: TokenStream) -> MacroResult {
    ShouldPanicCollector::extend_with_config_fn(args, item)
}

mod validate {
    use super::CairoString;
    use crate::attributes::{AttributeInfo, ErrorExt};
    use cairo_lang_macro::Diagnostic;
    use cairo_lang_syntax::node::{ast::Expr, db::SyntaxGroup};

    pub fn list_of_strings<T: AttributeInfo>(
        db: &dyn SyntaxGroup,
        expr: &Expr,
    ) -> Result<Vec<CairoString>, Diagnostic> {
        let mut strings = vec![];

        match expr {
            Expr::ShortString(string) => {
                let string = string.string_value(db).unwrap();

                strings.push(CairoString::Short(string));
            }
            Expr::String(string) => {
                let string = string.string_value(db).unwrap();

                strings.push(CairoString::Normal(string));
            }
            Expr::Tuple(expressions) => {
                for expression in &expressions.expressions(db).elements(db) {
                    match expression {
                        Expr::ShortString(string) => {
                            let string = string.string_value(db).unwrap();

                            strings.push(CairoString::Short(string));
                        }
                        Expr::String(string) => {
                            let string = string.string_value(db).unwrap();

                            strings.push(CairoString::Normal(string));
                        }
                        _ => Err(T::error(format!(
                            "<expected> argument must be in form: {}",
                            T::ARGS_FORM
                        )))?,
                    }
                }
            }
            _ => Err(T::error(format!(
                "<expected> argument must be in form: {}",
                T::ARGS_FORM
            )))?,
        };

        Ok(strings)
    }
}
