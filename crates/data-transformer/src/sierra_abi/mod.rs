use anyhow::{bail, Result};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::Expr;
use data_representation::AllowedCalldataArgument;
use starknet::core::types::contract::AbiEntry;

mod complex_types;
pub(super) mod data_representation;
mod literals;
mod macros;
pub(super) mod parsing;

/// A main trait that allows particular calldata types to be recognized and transformed
trait SupportedCalldataKind {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument>;
}

/// A main function that transforms expressions supported by the transformer
/// to their correspondning serializable struct representations
pub(super) fn build_representation(
    expression: Expr,
    expected_type: &str,
    abi: &[AbiEntry],
    db: &SimpleParserDatabase,
) -> Result<AllowedCalldataArgument> {
    match expression {
        Expr::StructCtorCall(item) => item.transform(expected_type, abi, db),
        Expr::Literal(item) => item.transform(expected_type, abi, db),
        Expr::Unary(item) => item.transform(expected_type, abi, db),
        Expr::ShortString(item) => item.transform(expected_type, abi, db),
        Expr::String(item) => item.transform(expected_type, abi, db),
        Expr::True(item) => item.transform(expected_type, abi, db),
        Expr::False(item) => item.transform(expected_type, abi, db),
        Expr::Path(item) => item.transform(expected_type, abi, db),
        Expr::FunctionCall(item) => item.transform(expected_type, abi, db),
        Expr::InlineMacro(item) => item.transform(expected_type, abi, db),
        Expr::Tuple(item) => item.transform(expected_type, abi, db),
        Expr::Parenthesized(_)
        | Expr::Binary(_)
        | Expr::Block(_)
        | Expr::Match(_)
        | Expr::If(_)
        | Expr::Loop(_)
        | Expr::While(_)
        | Expr::For(_)
        | Expr::Closure(_)
        | Expr::ErrorPropagate(_)
        | Expr::FieldInitShorthand(_)
        | Expr::Indexed(_)
        | Expr::FixedSizeArray(_)
        | Expr::Missing(_) => {
            bail!(r#"Invalid argument type: unsupported expression for type "{expected_type}""#)
        }
    }
}
