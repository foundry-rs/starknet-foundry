use crate::{
    attributes::{AttributeInfo, ErrorExt},
    cairo_expression::CairoExpression,
};
use cairo_lang_macro::Diagnostic;
use cairo_lang_syntax::node::{ast::Expr, db::SyntaxGroup};
use num_bigint::BigInt;
use url::Url;

pub trait ParseFromExpr<E>: Sized {
    fn parse_from_expr<T: AttributeInfo>(
        db: &dyn SyntaxGroup,
        expr: &E,
        arg_name: &str,
    ) -> Result<Self, Diagnostic>;
}

#[derive(Debug, Clone)]
pub enum Felt {
    Number(Number),
    ShortString(ShortString),
}

impl CairoExpression for Felt {
    fn as_cairo_expression(&self) -> String {
        match self {
            Self::Number(number) => number.as_cairo_expression(),
            Self::ShortString(string) => string.as_cairo_expression(),
        }
    }
}

impl ParseFromExpr<Expr> for Felt {
    fn parse_from_expr<T: AttributeInfo>(
        db: &dyn SyntaxGroup,
        expr: &Expr,
        arg_name: &str,
    ) -> Result<Self, Diagnostic> {
        match expr {
            Expr::ShortString(string) => {
                let string = string.string_value(db).unwrap();

                Ok(Self::ShortString(ShortString(string)))
            }
            Expr::Literal(string) => {
                let num = string.numeric_value(db).unwrap();

                Ok(Self::Number(Number(num)))
            }
            _ => Err(T::error(format!("<{arg_name}> argument must be felt")))?,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Number(pub(crate) BigInt);

#[derive(Debug, Clone)]
pub struct ShortString(pub(crate) String);

impl CairoExpression for Number {
    fn as_cairo_expression(&self) -> String {
        format!("0x{}", self.0.to_str_radix(16))
    }
}

impl CairoExpression for ShortString {
    fn as_cairo_expression(&self) -> String {
        format!("'{}'", self.0)
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
                    .ok_or_else(|| T::error(format!("<{arg_name}> got invalid number literal")))?;

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
impl ParseFromExpr<Expr> for ShortString {
    fn parse_from_expr<T: AttributeInfo>(
        db: &dyn SyntaxGroup,
        expr: &Expr,
        arg_name: &str,
    ) -> Result<Self, Diagnostic> {
        match expr {
            Expr::ShortString(string) => match string.string_value(db) {
                None => Err(T::error(format!("<{arg_name}> is not a valid string"))),
                Some(string) => Ok(ShortString(string)),
            },
            _ => Err(T::error(format!(
                "<{arg_name}> invalid type, should be: double quotted string"
            ))),
        }
    }
}

impl CairoExpression for String {
    fn as_cairo_expression(&self) -> String {
        format!(r#""{self}""#)
    }
}

impl CairoExpression for Url {
    fn as_cairo_expression(&self) -> String {
        format!(r#""{self}""#)
    }
}
