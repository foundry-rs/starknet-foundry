mod sierra_abi;

use crate::shared::formatting::format_passed_vs_expected;
use crate::shared::parsing::parse_expression;
use crate::shared::{extraction::extract_function_from_selector, formatting::format_abi_members};
use crate::transformer::sierra_abi::build_representation;
use anyhow::{Context, Result, bail};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_syntax::node::ast::Expr;
use conversions::serde::serialize::SerializeToFeltVec;
use itertools::Itertools;
use starknet_rust::core::types::contract::{AbiEntry, AbiFunction};
use starknet_types_core::felt::Felt;

/// Interpret `calldata` as a comma-separated series of expressions in Cairo syntax and serialize it
pub fn transform(calldata: &str, abi: &[AbiEntry], function_selector: &Felt) -> Result<Vec<Felt>> {
    let function = extract_function_from_selector(abi, *function_selector).with_context(|| {
        format!(
            r#"Function with selector "{function_selector:#x}" not found in ABI of the contract"#
        )
    })?;

    let db = SimpleParserDatabase::default();

    let input = convert_to_tuple(calldata);
    let calldata = split_expressions(&input, &db)?;

    process(&calldata, &function, abi, &db).context("Error while processing Cairo-like calldata")
}

fn split_expressions<'a>(input: &'a str, db: &'a SimpleParserDatabase) -> Result<Vec<Expr<'a>>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }
    let expr = parse_expression(input, db)?;

    match expr {
        Expr::Tuple(tuple) => Ok(tuple.expressions(db).elements(db).collect()),
        Expr::Parenthesized(expr) => Ok(vec![expr.expr(db)]),
        _ => bail!("Wrong calldata format - expected tuple of Cairo expressions"),
    }
}

fn process(
    calldata: &[Expr],
    function: &AbiFunction,
    abi: &[AbiEntry],
    db: &SimpleParserDatabase,
) -> Result<Vec<Felt>> {
    let n_inputs = function.inputs.len();
    let n_arguments = calldata.len();

    if n_arguments != n_inputs {
        bail!(format_invalid_args_number_error(
            calldata,
            function,
            n_arguments,
            db
        ));
    }

    function
        .inputs
        .iter()
        .zip(calldata)
        .map(|(parameter, expr)| {
            let representation = build_representation(expr.clone(), &parameter.r#type, abi, db)?;
            Ok(representation.serialize_to_vec())
        })
        .flatten_ok()
        .collect::<Result<_>>()
}

fn format_invalid_args_number_error(
    calldata: &[Expr],
    function: &AbiFunction,
    n_arguments: usize,
    db: &SimpleParserDatabase,
) -> String {
    let n_inputs = function.inputs.len();
    let passed = calldata
        .iter()
        .map(|expr| {
            expr.as_syntax_node()
                .get_text_without_trivia(db)
                .to_string(db)
        })
        .join(", ");

    format_passed_vs_expected(
        format!(
            "Invalid number of arguments for {}: passed {n_arguments}, expected {n_inputs}",
            function.name
        ),
        format!("({passed})"),
        format!("({})", format_abi_members(&function.inputs)),
    )
}

fn convert_to_tuple(calldata: &str) -> String {
    // We need to convert our comma-separated string of expressions into something that is a valid
    // Cairo expression, so we can parse it.
    //
    // We convert to tuple by wrapping in `()` with a trailing `,` to handle case of a single argument
    if calldata.is_empty() {
        return String::new();
    }
    format!("({calldata},)")
}
