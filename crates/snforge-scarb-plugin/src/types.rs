use crate::{
    attributes::{AttributeInfo, ErrorExt},
    cairo_expression::CairoExpression,
};
use cairo_lang_macro::Diagnostic;
use cairo_lang_syntax::node::{ast::Expr, db::SyntaxGroup};
use url::Url;

pub trait ParseFromExpr<E>: Sized {
    fn parse_from_expr<T: AttributeInfo>(
        db: &dyn SyntaxGroup,
        expr: &E,
        arg_name: &str,
    ) -> Result<Self, Diagnostic>;
}

#[derive(Debug, Clone)]
pub struct Number(String);

impl CairoExpression for Number {
    fn as_cairo_expression(&self) -> String {
        format!("0x{}", self.0)
    }
}

impl ParseFromExpr<Expr> for Number {
    fn parse_from_expr<T: AttributeInfo>(
        db: &dyn SyntaxGroup,
        expr: &Expr,
        arg_name: &str,
    ) -> Result<Self, Diagnostic> {
        match expr {
            Expr::Literal(literal) => {
                let num = literal
                    .numeric_value(db)
                    .ok_or_else(|| T::error(format!("<{arg_name}> got invalid number literal")))?
                    .to_str_radix(16);

                Ok(Self(num))
            }
            _ => Err(T::error(format!("<{arg_name}> should be number literal",))),
        }
    }
}

impl ParseFromExpr<Expr> for Url {
    fn parse_from_expr<T: AttributeInfo>(
        db: &dyn SyntaxGroup,
        expr: &Expr,
        arg_name: &str,
    ) -> Result<Self, Diagnostic> {
        let url = String::parse_from_expr::<T>(db, expr, arg_name)?;

        Url::parse(&url).map_err(|_| T::error(format!("<{arg_name}> is not a valid url")))
    }
}

impl ParseFromExpr<Expr> for String {
    fn parse_from_expr<T: AttributeInfo>(
        db: &dyn SyntaxGroup,
        expr: &Expr,
        arg_name: &str,
    ) -> Result<Self, Diagnostic> {
        match expr {
            Expr::String(string) => match string.string_value(db) {
                None => Err(T::error(format!("<{arg_name}> is not a valid string"))),
                Some(string) => Ok(string),
            },
            _ => Err(T::error(format!(
                "<{arg_name}> invalid type, should be: double quotted string"
            ))),
        }
    }
}
