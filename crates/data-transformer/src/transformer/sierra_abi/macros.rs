use super::data_representation::{AllowedCalldataArgument, CalldataArrayMacro};
use super::parsing::parse_inline_macro;
use super::{SupportedCalldataKind, build_representation};
use crate::shared::parsing::parse_expression;
use anyhow::{Context, Result, bail};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_syntax::node::ast::{
    Expr, ExprInlineMacro, GenericArgValue, PathSegment, PathSegment::Simple,
};
use itertools::Itertools;
use starknet::core::types::contract::AbiEntry;

impl SupportedCalldataKind for ExprInlineMacro {
    fn transform(
        &self,
        expected_type: &str,
        abi: &[AbiEntry],
        db: &SimpleParserDatabase,
    ) -> Result<AllowedCalldataArgument> {
        // array![] calls
        let parsed_exprs = parse_inline_macro(self, db)?;

        // We do not expect any other expression in proper ABI
        let Expr::Path(path) = parse_expression(expected_type, db)? else {
            bail!(
                "Unexpected expression encountered in ABI: {}. ABI may be invalid",
                expected_type
            );
        };

        let type_parameters_from_abi = path
            .elements(db)
            .into_iter()
            .find_map(|element| match element {
                // We expect exactly one PathSegment::WithGenericArgs. More means that ABI is broken, less means that type other than Array is expected
                Simple(_) => None,
                PathSegment::WithGenericArgs(segment) => Some(
                                segment
                                    .generic_args(db)
                                    .generic_args(db)
                                    .elements(db)
                                    .into_iter()
                                    .map(|arg| match arg {
                                        // There shouldn't be expressions like `identifier<T: some-trait-bound>` in the ABI
                                        arg @ cairo_lang_syntax::node::ast::GenericArg::Named(_) => bail!(
                                            "Unexpected named generic found in ABI: {}. Contract ABI may be invalid",
                                            arg.as_syntax_node().get_text(db)
                                        ),
                                        cairo_lang_syntax::node::ast::GenericArg::Unnamed(arg) => {
                                            match arg.value(db) {
                                                GenericArgValue::Expr(expr) => {
                                                    Ok(expr.as_syntax_node().get_text(db))
                                                }
                                                // Placeholder parameters are not allowed in ABI too
                                                value @ GenericArgValue::Underscore(_) => bail!(
                                                    "Unexpected type with underscore generic placeholder found in ABI: {}. Contract ABI may be invalid",
                                                    value.as_syntax_node().get_text(db)
                                                ),
                                            }
                                        }
                                    })
                                    .collect::<Result<Vec<_>>>(),
                            ),
                PathSegment::Missing(_path_segment_missing) => {
                    // TODO: Handle path_segment_missing
                    None
                },
            })
            .transpose()?
            .with_context(|| format!(r#"Invalid argument type, expected "{expected_type}", got array"#))?;

        // Check by string; A proper array type in ABI looks exactly like this
        if !expected_type.contains("core::array::Array") {
            bail!(r#"Expected "{}", got array"#, expected_type);
        }

        // Array should have exactly one type parameter. ABI is invalid otherwise
        let [element_type] = &type_parameters_from_abi[..] else {
            let parameters_punctuated = type_parameters_from_abi.into_iter().join(", ");

            bail!(
                "Expected exactly one generic parameter of Array type, got {parameters_punctuated}. Contract ABI may be invalid",
            );
        };

        let arguments = parsed_exprs
            .into_iter()
            .map(|arg| build_representation(arg, element_type, abi, db))
            .collect::<Result<Vec<AllowedCalldataArgument>>>()?;

        Ok(AllowedCalldataArgument::ArrayMacro(
            CalldataArrayMacro::new(arguments),
        ))
    }
}
