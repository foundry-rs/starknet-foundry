use super::ShouldPanicCollector;
use crate::{
    attributes::{AttributeInfo, ErrorExt},
    cairo_expression::CairoExpression,
    types::{Felt, ParseFromExpr},
};
use cairo_lang_macro::{quote, Diagnostic, TokenStream};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::{ast::Expr, Terminal};

#[derive(Debug, Clone, Default)]
pub enum Expected {
    Felt(Felt),
    ByteArray(String),
    Array(Vec<Felt>),
    #[default]
    Any,
}

impl CairoExpression for Expected {
    fn as_cairo_expression(&self) -> TokenStream {
        match self {
            Self::Felt(felt) => {
                let string = felt.as_cairo_expression();

                quote!(snforge_std::_internals::config_types::Expected::ShortString(#string))
            }
            Self::ByteArray(string) => {
                let string = string.as_cairo_expression();

                quote!(snforge_std::_internals::config_types::Expected::ByteArray(#string))
            }
            Self::Array(strings) => {
                let arr = strings.as_cairo_expression();

                quote!(snforge_std::_internals::config_types::Expected::Array(#arr))
            }
            Self::Any => quote!(snforge_std::_internals::config_types::Expected::Any),
        }
    }
}

impl ParseFromExpr<Expr> for Expected {
    fn parse_from_expr<T: AttributeInfo>(
        db: &SimpleParserDatabase,
        expr: &Expr,
        arg_name: &str,
    ) -> Result<Self, Diagnostic> {
        let error_msg = format!(
            "<{arg_name}> argument must be string, short string, number or list of short strings or numbers in regular brackets ()"
        );

        match expr {
            Expr::ShortString(_) | Expr::Literal(_) => {
                Ok(Self::Felt(
                    Felt::parse_from_expr::<ShouldPanicCollector>(db, expr, arg_name)
                        // this unwrap is safe because we checked if expression is valid short string or number
                        .unwrap(),
                ))
            }
            Expr::String(string) => {
                let string = string.text(db).trim_matches('"').to_string();

                Ok(Self::ByteArray(string))
            }
            Expr::Tuple(expressions) => {
                let elements = expressions
                    .expressions(db)
                    .elements(db)
                    .map(|expr| Felt::parse_from_expr::<ShouldPanicCollector>(db, &expr, arg_name))
                    .collect::<Result<Vec<Felt>, Diagnostic>>()
                    .map_err(|_| ShouldPanicCollector::error(error_msg))?;

                Ok(Self::Array(elements))
            }
            _ => Err(ShouldPanicCollector::error(error_msg)),
        }
    }
}
