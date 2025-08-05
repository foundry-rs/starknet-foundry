mod sierra_abi;

use crate::shared::extraction::extract_function_from_selector;
use crate::shared::parsing::parse_expression;
use crate::transformer::sierra_abi::build_representation;
use anyhow::{Context, Result, bail, ensure};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::Expr;
use conversions::serde::serialize::SerializeToFeltVec;
use itertools::Itertools;
use starknet::core::types::Felt;
use starknet::core::types::contract::{AbiEntry, AbiFunction};

/// Interpret `calldata` as a comma-separated series of expressions in Cairo syntax and serialize it
pub fn transform(calldata: &str, abi: &[AbiEntry], function_selector: &Felt) -> Result<Vec<Felt>> {
    let function = extract_function_from_selector(abi, *function_selector).with_context(|| {
        format!(
            r#"Function with selector "{function_selector:#x}" not found in ABI of the contract"#
        )
    })?;

    let db = SimpleParserDatabase::default();

    let calldata = split_expressions(calldata, &db)?;

    process(calldata, &function, abi, &db).context("Error while processing Cairo-like calldata")
}

fn split_expressions(input: &str, db: &SimpleParserDatabase) -> Result<Vec<Expr>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }
    // We need to convert our comma-separated string of expressions into something that is a valid
    // Cairo expression, so we can parse it.
    //
    // We convert to tuple by wrapping in `()` with a trailing `,` to handle case of a single argument
    let input = format!("({input},)");
    let expr = parse_expression(&input, db)?;

    match expr {
        Expr::Tuple(tuple) => Ok(tuple.expressions(db).elements(db).collect()),
        Expr::Parenthesized(expr) => Ok(vec![expr.expr(db)]),
        _ => bail!("Wrong calldata format - expected tuple of Cairo expressions"),
    }
}

fn process(
    calldata: Vec<Expr>,
    function: &AbiFunction,
    abi: &[AbiEntry],
    db: &SimpleParserDatabase,
) -> Result<Vec<Felt>> {
    let n_inputs = function.inputs.len();
    let n_arguments = calldata.len();

    ensure!(
        n_inputs == n_arguments,
        "Invalid number of arguments: passed {}, expected {}",
        n_arguments,
        n_inputs,
    );

    function
        .inputs
        .iter()
        .zip(calldata)
        .map(|(parameter, expr)| {
            let representation = build_representation(expr, &parameter.r#type, abi, db)?;
            Ok(representation.serialize_to_vec())
        })
        .flatten_ok()
        .collect::<Result<_>>()
}
