use crate::utils::create_single_token;
use crate::{
    attributes::{AttributeInfo, ErrorExt},
    cairo_expression::CairoExpression,
};
use cairo_lang_macro::{Diagnostic, TokenStream};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::{ast::Expr, Terminal};
use num_bigint::BigInt;
use url::Url;

pub trait ParseFromExpr<E>: Sized {
    fn parse_from_expr<T: AttributeInfo>(
        db: &SimpleParserDatabase,
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
    fn as_cairo_expression(&self) -> TokenStream {
        match self {
            Self::Number(number) => number.as_cairo_expression(),
            Self::ShortString(string) => string.as_cairo_expression(),
        }
    }
}

impl ParseFromExpr<Expr> for Felt {
    fn parse_from_expr<T: AttributeInfo>(
        db: &SimpleParserDatabase,
        expr: &Expr,
        arg_name: &str,
    ) -> Result<Self, Diagnostic> {
        match expr {
            Expr::ShortString(string) => {
                let string = string.text(db).trim_matches('\'').to_string();
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Number(pub(crate) BigInt);

impl Number {
    pub fn validate_in_gas_range<T: AttributeInfo>(
        &self,
        arg_name: &str,
    ) -> Result<(), Diagnostic> {
        let max = u64::MAX;
        if *self > Number(max.into()) {
            return Err(T::error(format!(
                "{arg_name} it too large (max permissible value is {max})"
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ShortString(pub(crate) String);

impl CairoExpression for Number {
    fn as_cairo_expression(&self) -> TokenStream {
        TokenStream::new(vec![create_single_token(format!(
            "0x{}",
            self.0.to_str_radix(16)
        ))])
    }
}

impl CairoExpression for ShortString {
    fn as_cairo_expression(&self) -> TokenStream {
        TokenStream::new(vec![create_single_token(format!("'{}'", self.0))])
    }
}

impl ParseFromExpr<Expr> for Number {
    fn parse_from_expr<T: AttributeInfo>(
        db: &SimpleParserDatabase,
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
        db: &SimpleParserDatabase,
        expr: &Expr,
        arg_name: &str,
    ) -> Result<Self, Diagnostic> {
        let url = String::parse_from_expr::<T>(db, expr, arg_name)?;

        Url::parse(&url).map_err(|_| T::error(format!("<{arg_name}> is not a valid url")))
    }
}

impl ParseFromExpr<Expr> for String {
    fn parse_from_expr<T: AttributeInfo>(
        db: &SimpleParserDatabase,
        expr: &Expr,
        arg_name: &str,
    ) -> Result<Self, Diagnostic> {
        match expr {
            Expr::String(string) => Ok(string.text(db).trim_matches('"').to_string()),
            _ => Err(T::error(format!(
                "<{arg_name}> invalid type, should be: double quotted string"
            ))),
        }
    }
}

impl ParseFromExpr<Expr> for ShortString {
    fn parse_from_expr<T: AttributeInfo>(
        db: &SimpleParserDatabase,
        expr: &Expr,
        arg_name: &str,
    ) -> Result<Self, Diagnostic> {
        match expr {
            Expr::ShortString(string) => {
                let string = string.text(db).trim_matches('\'').to_string();
                Ok(ShortString(string))
            }
            _ => Err(T::error(format!(
                "<{arg_name}> invalid type, should be: double quotted string"
            ))),
        }
    }
}

impl CairoExpression for String {
    fn as_cairo_expression(&self) -> TokenStream {
        TokenStream::new(vec![create_single_token(format!(r#""{self}""#))])
    }
}

impl CairoExpression for Url {
    fn as_cairo_expression(&self) -> TokenStream {
        TokenStream::new(vec![create_single_token(format!(r#""{self}""#))])
    }
}
