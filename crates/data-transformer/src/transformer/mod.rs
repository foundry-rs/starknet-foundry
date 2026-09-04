mod sierra_abi;

use crate::shared::extraction::extract_function_from_selector;
use crate::shared::formatting::{ArgumentListKind, format_abi_members, format_passed_vs_expected};
use crate::shared::parsing::parse_expression;
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
        bail!(format_invalid_args_error(
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

fn format_invalid_args_error(
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
        .collect::<Vec<_>>();
    let expected = format_abi_members(&function.inputs);
    let diagnostics = format_positional_argument_diagnostics(function, n_arguments);

    format_passed_vs_expected(
        format!(
            "Invalid arguments for function `{}`: passed {n_arguments}, expected {n_inputs}",
            function.name
        ),
        &diagnostics,
        &passed,
        &expected,
        ArgumentListKind::Positional,
    )
}

fn format_positional_argument_diagnostics(
    function: &AbiFunction,
    n_arguments: usize,
) -> Vec<String> {
    if n_arguments < function.inputs.len() {
        function
            .inputs
            .iter()
            .enumerate()
            .skip(n_arguments)
            .map(|(position, argument)| {
                format!(
                    "missing argument `{}` at position {}",
                    argument.name,
                    position + 1
                )
            })
            .collect()
    } else {
        (function.inputs.len()..n_arguments)
            .map(|position| format!("unexpected argument at position {}", position + 1))
            .collect()
    }
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
