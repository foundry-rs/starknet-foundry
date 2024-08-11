use crate::handle_rpc_error;
use anyhow::{bail, Context, Result};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{
    Expr, FunctionWithBody, ModuleItem, Statement, StatementExpr, SyntaxFile,
};
use cairo_lang_syntax::node::{SyntaxNode, TypedSyntaxNode};
use starknet::core::types::contract::{AbiEntry, AbiFunction, StateMutability};
use starknet::core::types::{BlockId, BlockTag, ContractClass, Felt};
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use std::collections::HashMap;

/// Parses input calldata and puts inside wrapper Cairo code to allow parsing by [`SimpleParserDatabase`]
fn parse_input_calldata(input_calldata: &str, db: &SimpleParserDatabase) -> Result<SyntaxNode> {
    let input_calldata = input_calldata
        .strip_prefix("{")
        .context("Couldn't parse input calldata, missing {")?
        .strip_suffix("}")
        .context("Couldn't parse input calldata, missing }")?;

    let temporary_code = format!("fn __WRAPPER_FUNC_TO_PARSE__() {{ ({input_calldata}); }}");
    let (node, diagnostics) = db.parse_virtual_with_diagnostics(temporary_code);

    match diagnostics.check_error_free() {
        Ok(()) => Ok(node),
        Err(_) => {
            bail!(
                "Invalid Cairo expression found in input calldata:\n{}",
                diagnostics.format(db)
            )
        }
    }
}

/// Traverses through AST to get parenthesised expression with calldata
fn get_input_expr_between_parentheses(node: SyntaxNode, db: &SimpleParserDatabase) -> Expr {
    let syntax_file = SyntaxFile::from_syntax_node(db, node);
    let module_item_list = syntax_file.items(db);
    let function_with_body = module_item_list
        .elements(db)
        .into_iter()
        .filter_map(|x| match x {
            ModuleItem::FreeFunction(f) => Some(f),
            _ => None,
        })
        .collect::<Vec<FunctionWithBody>>()
        .pop()
        .expect("Failed to parse wrapper calldata function");
    let expr_block = function_with_body.body(db);
    let statement_list = expr_block.statements(db);
    let statement_expr = statement_list
        .elements(db)
        .into_iter()
        .filter_map(|x| match x {
            Statement::Expr(expr) => Some(expr),
            _ => None,
        })
        .collect::<Vec<StatementExpr>>()
        .pop()
        .expect("Failed to parse wrapper calldata function");
    statement_expr.expr(db)
}

/// Gets input expression artificially put between parentheses in [`parse_input_calldata`]
fn get_expr_list(parsed_node: SyntaxNode, db: &SimpleParserDatabase) -> Vec<Expr> {
    let statement_list_node = get_input_expr_between_parentheses(parsed_node, db);
    // TODO remove comment
    // Possibilities:
    // 123 - ExprParenthesized ( TerminalLiteralNumber )
    // 123, 123 - ExprListParenthesized ( ExprList )
    // (123) - ExprParenthesized ( ExprParenthesized )
    // (123, 123) - ExprParenthesized ( ExprListParenthesized )
    // 123, (123) - ExprListParenthesized ( ExprList )
    match statement_list_node {
        // List of arguments - function accepts more than one argument
        Expr::Tuple(list_of_args) => list_of_args.expressions(db).elements(db),
        // Single argument between parentheses
        Expr::Parenthesized(single_arg) => {
            vec![single_arg.expr(db)]
        }
        _ => {
            unreachable!(
                "Due to construction of the wrapper function, other possibilities are not possible"
            )
        }
    }
}
fn map_functions_to_selectors(abi: &[AbiEntry]) -> HashMap<Felt, AbiFunction> {
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
                // Turn constructor into AbiFunction to simplify searching for function in
                // `transform_input_calldata`
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
                map.extend(map_functions_to_selectors(&interface.items));
            }
            _ => {}
        }
    }
    map
}

pub async fn transform_input_calldata(
    input_calldata: &str,
    function_selector: &Felt,
    class_hash: Felt,
    client: &JsonRpcClient<HttpTransport>,
) -> Result<Vec<Felt>> {
    let db = SimpleParserDatabase::default();

    // TODO handle when parsing fails and fn __WRAPPER_FUNC_TO_PARSE__() is printed to stderr
    let parsed_node = parse_input_calldata(input_calldata, &db)?;
    let contract_class = client
        .get_class(BlockId::Tag(BlockTag::Latest), class_hash)
        .await
        .map_err(handle_rpc_error)
        .context(format!(
            "Couldn't retrieve contract class with hash: {class_hash:#x}"
        ))?;

    let arguments_expr_list = get_expr_list(parsed_node, &db);

    match contract_class {
        ContractClass::Sierra(sierra_class) => {
            let abi: Vec<AbiEntry> = serde_json::from_str(sierra_class.abi.as_str())
                .context("Couldn't deserialize ABI received from chain")?;
            let selector_function_map = map_functions_to_selectors(&abi);
            let called_function = selector_function_map
                .get(function_selector)
                .context(format!(
                    r#"Function with selector "{function_selector}" not found in ABI of the contract"#
                ))?;

            if called_function.inputs.len() != arguments_expr_list.len() {
                bail!(
                    "Invalid number of arguments, passed {}, expected {}",
                    arguments_expr_list.len(),
                    called_function.inputs.len()
                )
            }

            todo!();
        }
        ContractClass::Legacy(_legacy_class) => {
            todo!("Finish adding legacy ABI handling");
        }
    };
    todo!()
}
