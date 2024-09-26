use crate::helpers::data_transformer::sierra_abi::build_representation;
use anyhow::{bail, ensure, Context, Result};
use cairo_lang_diagnostics::DiagnosticsBuilder;
use cairo_lang_filesystem::ids::{FileKind, FileLongId, VirtualFile};
use cairo_lang_parser::parser::Parser;
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::Expr;
use cairo_lang_utils::Intern;
use conversions::serde::serialize::SerializeToFeltVec;
use itertools::Itertools;
use starknet::core::types::contract::{AbiEntry, AbiFunction, StateMutability};
use starknet::core::types::{ContractClass, Felt};
use starknet::core::utils::get_selector_from_name;
use std::collections::HashMap;

pub fn transform(
    calldata: &Vec<String>,
    class_definition: ContractClass,
    function_selector: &Felt,
) -> Result<Vec<Felt>> {
    let sierra_class = match class_definition {
        ContractClass::Sierra(class) => class,
        ContractClass::Legacy(_) => {
            bail!("Transformation of Cairo-like expressions is not available for Cairo0 contracts")
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

    let result_for_cairo_like = process_as_cairo_expressions(calldata, function, &abi, &db)
        .context("Error while processing Cairo-like calldata");

    if result_for_cairo_like.is_ok() {
        return result_for_cairo_like;
    }

    let result_for_already_serialized = process_as_serialized(calldata, &abi, &db)
        .context("Error while processing serialized calldata");

    match result_for_already_serialized {
        Err(_) => result_for_cairo_like,
        ok => ok,
    }
}

fn process_as_cairo_expressions(
    calldata: &Vec<String>,
    function: &AbiFunction,
    abi: &Vec<AbiEntry>,
    db: &SimpleParserDatabase,
) -> Result<Vec<Felt>> {
    let n_inputs = function.inputs.len();
    let n_arguments = calldata.len();

    ensure!(
        n_inputs == n_arguments,
        "Invalid number of arguments: passed {}, expected {}",
        n_inputs,
        n_arguments
    );

    function
        .inputs
        .iter()
        .zip(calldata)
        .map(|(parameter, value)| {
            let expr = parse(value, &db)?;
            let representation = build_representation(expr, &parameter.r#type, &abi, &db)?;
            Ok(representation.serialize_to_vec())
        })
        .flatten_ok()
        .collect::<Result<_>>()
}

fn process_as_serialized(
    calldata: &Vec<String>,
    abi: &Vec<AbiEntry>,
    db: &SimpleParserDatabase,
) -> Result<Vec<Felt>> {
    calldata
        .iter()
        .map(|expression| {
            let expr = parse(expression, db)?;
            let representation = build_representation(expr, "felt252", abi, db)?;
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
                // Transparency of constructors and other functions
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
            _ => {}
        }
    }

    map
}

fn parse(source: &str, db: &SimpleParserDatabase) -> Result<Expr> {
    let file = FileLongId::Virtual(VirtualFile {
        parent: None,
        name: "parser_input".into(),
        content: source.to_string().into(),
        code_mappings: [].into(),
        kind: FileKind::Expr,
    })
    .intern(db);

    let mut diagnostics = DiagnosticsBuilder::default();
    let expression = Parser::parse_file_expr(db, &mut diagnostics, file, source);
    let diagnostics = diagnostics.build();

    if diagnostics.check_error_free().is_err() {
        bail!(
            "Invalid Cairo expression found in input calldata \"{}\":\n{}",
            source,
            diagnostics.format(db)
        )
    }

    Ok(expression)
}
