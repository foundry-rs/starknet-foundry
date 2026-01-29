use super::SupportedCalldataKind;
use super::data_representation::{AllowedCalldataArgument, CalldataPrimitive};
use anyhow::{Context, Result, bail};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::Terminal;
use cairo_lang_syntax::node::ast::{
    Expr, ExprUnary, TerminalFalse, TerminalLiteralNumber, TerminalShortString, TerminalString,
    TerminalTrue, UnaryOperator,
};
use starknet_rust::core::types::contract::AbiEntry;
use std::ops::Neg;

impl SupportedCalldataKind for TerminalLiteralNumber<'_> {
    fn transform(
        &self,
        expected_type: &str,
        _abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        let (value, suffix) = self
            .numeric_value_and_suffix(db)
            .with_context(|| format!("Couldn't parse value: {}", self.text(db).to_string(db)))?;

        let proper_param_type = match suffix {
            None => expected_type,
            Some(ref suffix) => &suffix.to_string(db),
        };

        Ok(AllowedCalldataArgument::Primitive(
            CalldataPrimitive::try_from_str_with_type(&value.to_string(), proper_param_type)?,
        ))
    }
}

impl SupportedCalldataKind for ExprUnary<'_> {
    fn transform(
        &self,
        expected_type: &str,
        _abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        let (value, suffix) = match self.expr(db) {
            Expr::Literal(literal_number) => literal_number
                .numeric_value_and_suffix(db)
                .with_context(|| {
                    format!(
                        "Couldn't parse value: {}",
                        literal_number.text(db).to_string(db)
                    )
                }),
            _ => bail!("Invalid expression with unary operator, only numbers allowed"),
        }?;

        let proper_param_type = match suffix {
            None => expected_type,
            Some(ref suffix) => &suffix.to_string(db),
        };

        match self.op(db) {
            UnaryOperator::Not(_) => {
                bail!("Invalid unary operator in expression !{value} , only - allowed, got !",)
            }
            UnaryOperator::Desnap(_) => {
                bail!("Invalid unary operator in expression *{value} , only - allowed, got *",)
            }
            UnaryOperator::BitNot(_) => {
                bail!("Invalid unary operator in expression ~{value} , only - allowed, got ~",)
            }
            UnaryOperator::At(_) => {
                bail!("Invalid unary operator in expression @{value} , only - allowed, got @",)
            }
            // TODO: Check if this is correct for `UnaryOperator::Reference`
            UnaryOperator::Minus(_) | UnaryOperator::Reference(_) => {}
        }

        Ok(AllowedCalldataArgument::Primitive(
            CalldataPrimitive::try_from_str_with_type(&value.neg().to_string(), proper_param_type)?,
        ))
    }
}

impl SupportedCalldataKind for TerminalShortString<'_> {
    fn transform(
        &self,
        expected_type: &str,
        _abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        let value = self
            .string_value(db)
            .context("Invalid shortstring passed as an argument")?;

        // TODO(#2623) add better handling
        let expected_type = match expected_type.split("::").last() {
            Some("felt" | "felt252") => "shortstring",
            _ => expected_type,
        };

        Ok(AllowedCalldataArgument::Primitive(
            CalldataPrimitive::try_from_str_with_type(&value, expected_type)?,
        ))
    }
}

impl SupportedCalldataKind for TerminalString<'_> {
    fn transform(
        &self,
        expected_type: &str,
        _abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        let value = self
            .string_value(db)
            .context("Invalid string passed as an argument")?;

        Ok(AllowedCalldataArgument::Primitive(
            CalldataPrimitive::try_from_str_with_type(&value, expected_type)?,
        ))
    }
}

impl SupportedCalldataKind for TerminalTrue<'_> {
    fn transform(
        &self,
        expected_type: &str,
        _abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        let value = self.text(db).to_string(db);

        Ok(AllowedCalldataArgument::Primitive(
            CalldataPrimitive::try_from_str_with_type(&value, expected_type)?,
        ))
    }
}

impl SupportedCalldataKind for TerminalFalse<'_> {
    fn transform(
        &self,
        expected_type: &str,
        _abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        let value = self.text(db).to_string(db);

        Ok(AllowedCalldataArgument::Primitive(
            CalldataPrimitive::try_from_str_with_type(&value, expected_type)?,
        ))
    }
}
