use super::{AttributeInfo, AttributeTypeData};
use crate::{args::Arguments, attributes::AttributeCollector};
use cairo_lang_macro::Diagnostics;
use cairo_lang_syntax::node::db::SyntaxGroup;
use std::fmt::Display;

pub struct ShouldPanicCollector;

impl AttributeInfo for ShouldPanicCollector {
    const ATTR_NAME: &'static str = "should_panic";
    const ARGS_FORM: &'static str = "[<expected>: `ByteArray` | `felt252` | [`felt252`,]]";
}

impl AttributeTypeData for ShouldPanicCollector {
    const CHEATCODE_NAME: &'static str = "set_config_should_panic";
}

#[derive(Debug, Clone, Default)]
enum Expected {
    ShortString(String),
    ByteArray(String),
    Array(Vec<String>),
    #[default]
    Any,
}

impl Display for Expected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShortString(string) => write!(
                f,
                "snforge_std::_config_types::Expected::ShortString('{string}')"
            ),
            Self::ByteArray(string) => write!(
                f,
                r#"snforge_std::_config_types::Expected::ByteArray("{string}")"#
            ),
            Self::Array(strings) => {
                let arr = strings.join(",");

                write!(f, "snforge_std::_config_types::Expected::Array([{arr}])")
            }
            Self::Any => write!(f, "snforge_std::_config_types::Expected::Any"),
        }
    }
}

impl AttributeCollector for ShouldPanicCollector {
    fn args_into_config_expression(
        db: &dyn SyntaxGroup,
        args: Arguments,
    ) -> Result<String, Diagnostics> {
        let named_args = args.named_only::<Self>()?;

        let expected = named_args.as_once_optional("expected")?;

        let expected = expected
            .map(|expr| validate::expected_value::<Self>(db, expr))
            .transpose()?
            .unwrap_or_default();

        Ok(format!(
            "snforge_std::_config_types::ShouldPanicConfig {{ expected: {expected} }}"
        ))
    }
}

mod validate {
    use super::Expected;
    use crate::attributes::{AttributeInfo, ErrorExt};
    use cairo_lang_macro::Diagnostic;
    use cairo_lang_syntax::node::{ast::Expr, db::SyntaxGroup};

    pub fn expected_value<T: AttributeInfo>(
        db: &dyn SyntaxGroup,
        expr: &Expr,
    ) -> Result<Expected, Diagnostic> {
        match expr {
            Expr::ShortString(string) => {
                let string = string.string_value(db).unwrap();

                Ok(Expected::ShortString(string))
            }
            Expr::String(string) => {
                let string = string.string_value(db).unwrap();

                Ok(Expected::ByteArray(string))
            }
            Expr::Tuple(expressions) => {
                let elements = expressions
                    .expressions(db)
                    .elements(db)
                    .into_iter()
                    .map(|expression| -> Result<String, Diagnostic> {
                        match expression {
                            Expr::ShortString(string) => {
                                let string = string.string_value(db).unwrap();

                                Ok(string)
                            }
                            Expr::Literal(string) => {
                                let string = string.numeric_value(db).unwrap();

                                Ok(format!("0x{}", string.to_str_radix(16)))
                            }
                            _ => Err(T::error(format!(
                                "<expected> argument must be in form: {}",
                                T::ARGS_FORM
                            )))?,
                        }
                    })
                    .collect::<Result<Vec<String>, Diagnostic>>()?;

                Ok(Expected::Array(elements))
            }
            _ => Err(T::error(format!(
                "<expected> argument must be in form: {}",
                T::ARGS_FORM
            ))),
        }
    }
}
