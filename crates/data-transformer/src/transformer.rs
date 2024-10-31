use super::sierra_abi::{build_representation, parsing::parse_expression};
use anyhow::{bail, ensure, Context, Result};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::Expr;
use conversions::serde::serialize::SerializeToFeltVec;
use itertools::Itertools;
use starknet::core::types::contract::{AbiEntry, AbiFunction, StateMutability};
use starknet::core::types::{ContractClass, Felt};
use starknet::core::utils::get_selector_from_name;
use std::collections::HashMap;

/// Interpret `calldata` as a comma-separated series of expressions in Cairo syntax and serialize it
pub fn transform(
    calldata: &str,
    class_definition: ContractClass,
    function_selector: &Felt,
) -> Result<Vec<Felt>> {
    let sierra_class = match class_definition {
        ContractClass::Sierra(class) => class,
        ContractClass::Legacy(_) => {
            bail!("Transformation of arguments is not available for Cairo Zero contracts")
        }
    };

    let abi: Vec<AbiEntry> = serde_json::from_str(sierra_class.abi.as_str())
        .context("Couldn't deserialize ABI received from chain")?;

    let selector_function_map = map_selectors_to_functions(&abi);

    let function = selector_function_map
        .get(function_selector)
        .with_context(|| {
            format!(
                r#"Function with selector "{function_selector}" not found in ABI of the contract"#
            )
        })?;

    let db = SimpleParserDatabase::default();

    let calldata = split_expressions(calldata, &db)?;

    process(calldata, function, &abi, &db).context("Error while processing Cairo-like calldata")
}

fn split_expressions(input: &str, db: &SimpleParserDatabase) -> Result<Vec<Expr>> {
    // We need to convert our comma-separated string of expressions into something that is a valid
    // Cairo expression, so we can parse it.
    //
    // We convert to tuple by wrapping in `()` with a trailing `,` to handle case of a single argument
    let input = format!("({input},)");
    let expr = parse_expression(&input, db)?;

    match expr {
        Expr::Tuple(tuple) => Ok(tuple.expressions(db).elements(db)),
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

fn map_selectors_to_functions(abi: &[AbiEntry]) -> HashMap<Felt, AbiFunction> {
    let mut map = HashMap::new();

    for abi_entry in abi {
        match abi_entry {
            AbiEntry::Function(func) => {
                map.insert(
                    get_selector_from_name(func.name.as_str()).unwrap(),
                    func.clone(),
                );
            }
            AbiEntry::Constructor(constructor) => {
                // We treat constructor like a regular function
                // because it's searched for using Felt entrypoint selector, identically as functions.
                // Also, we don't need any constructor-specific properties, just argument types.
                map.insert(
                    get_selector_from_name(constructor.name.as_str()).unwrap(),
                    AbiFunction {
                        name: constructor.name.clone(),
                        inputs: constructor.inputs.clone(),
                        outputs: vec![],
                        state_mutability: StateMutability::View,
                    },
                );
            }
            AbiEntry::Interface(interface) => {
                map.extend(map_selectors_to_functions(&interface.items));
            }
            // We don't need any other items at this point
            _ => {}
        }
    }

    map
}
