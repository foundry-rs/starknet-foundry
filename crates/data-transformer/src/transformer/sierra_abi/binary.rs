use crate::transformer::sierra_abi::SupportedCalldataKind;
use crate::transformer::sierra_abi::data_representation::AllowedCalldataArgument;
use anyhow::{Result, bail, ensure};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{
    BinaryOperator, Expr, ExprBinary, ExprFunctionCall, PathSegment,
};
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode};
use starknet::core::types::contract::AbiEntry;

impl SupportedCalldataKind for ExprBinary {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        let op = self.op(db);
        let lhs = self.lhs(db);
        let rhs = self.rhs(db);

        if !matches!(op, BinaryOperator::Dot(_)) {
            let op = op.as_syntax_node().get_text_without_trivia(db);
            bail!(r#"Invalid operator, expected ".", got "{op}""#)
        }

        let Expr::InlineMacro(lhs) = lhs else {
            let lhs = lhs.as_syntax_node().get_text_without_trivia(db);
            bail!(r#"Only "array![]" is supported as left-hand side of "." operator, got "{lhs}""#);
        };

        let Expr::FunctionCall(rhs) = rhs else {
            let rhs = rhs.as_syntax_node().get_text_without_trivia(db);
            bail!(r#"Only calling ".span()" on "array![]" is supported, got "{rhs}""#);
        };

        assert_is_span(&rhs, db)?;
        let expected_type = expected_type.replace("Span", "Array");
        lhs.transform(&expected_type, abi, db)
    }
}

fn assert_is_span(expr: &ExprFunctionCall, db: &SimpleParserDatabase) -> Result<()> {
    match expr
        .path(db)
        .elements(db)
        .last()
        .expect("Function call must have a name")
    {
        PathSegment::Simple(simple) => {
            let function_name = simple.ident(db).text(db);
            ensure!(
                function_name == "span",
                r#"Invalid function name, expected "span", got "{function_name}""#
            );
            Ok(())
        }
        PathSegment::WithGenericArgs(_) => {
            bail!("Invalid path specified: generic args in function call not supported")
        }
        PathSegment::Missing(_segment) => {
            bail!("Path segment missing")
        }
    }
}
