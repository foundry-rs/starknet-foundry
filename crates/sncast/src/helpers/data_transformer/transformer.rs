use crate::handle_rpc_error;
use crate::helpers::data_transformer::sierra_abi::parse_expr;
use anyhow::{bail, Context, Result};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{
    Expr, FunctionWithBody, ModuleItem, Statement, StatementExpr, SyntaxFile,
};
use cairo_lang_syntax::node::{SyntaxNode, TypedSyntaxNode};
use conversions::byte_array::ByteArray;
use conversions::serde::serialize::{BufferWriter, CairoSerialize, SerializeToFeltVec};
use conversions::u256::CairoU256;
use num_bigint::BigUint;
use starknet::core::types::contract::{AbiEntry, AbiFunction, StateMutability};
use starknet::core::types::{BlockId, BlockTag, ContractClass, Felt};
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct CalldataStructField(AllowedCalldataArguments);

impl CalldataStructField {
    pub fn new(value: AllowedCalldataArguments) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub(crate) struct CalldataStruct(Vec<CalldataStructField>);

impl CalldataStruct {
    pub fn new(arguments: Vec<CalldataStructField>) -> Self {
        Self(arguments)
    }
}

#[derive(Debug)]
pub(crate) struct CalldataArrayMacro(Vec<AllowedCalldataArguments>);

impl CalldataArrayMacro {
    pub fn new(arguments: Vec<AllowedCalldataArguments>) -> Self {
        Self(arguments)
    }
}

#[derive(Debug)]
pub(crate) struct CalldataEnum {
    position: usize,
    argument: Option<Box<AllowedCalldataArguments>>,
}

impl CalldataEnum {
    pub fn new(position: usize, argument: Option<Box<AllowedCalldataArguments>>) -> Self {
        Self { position, argument }
    }
}

#[derive(Debug)]
pub(crate) enum CalldataSingleArgument {
    // felt252
    // i8 - i128
    // u8 - u128
    // u256
    // TODO u512
    // bool
    // shortstring
    // string (ByteArray)
    Felt(Felt),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256(CairoU256),
    Bool(bool),
    ByteArray(ByteArray),
}

fn single_value_parsing_error_msg(
    value: &str,
    parsing_type: &str,
    append_message: Option<String>,
) -> String {
    let mut message = format!(r#"Failed to parse value "{value}" into type "{parsing_type}""#);
    if let Some(append_msg) = append_message {
        message += append_msg.as_str();
    }
    message
}

macro_rules! parse_with_type {
    ($id:ident, $type:ty) => {
        $id.parse::<$type>()
            .context(single_value_parsing_error_msg($id, stringify!($type), None))?
    };
}
impl CalldataSingleArgument {
    pub(crate) fn try_new(type_str_with_path: &str, value: &str) -> Result<Self> {
        // TODO add all corelib types
        let type_str = type_str_with_path
            .split("::")
            .last()
            .context("Couldn't parse parameter type from ABI")?;
        match type_str {
            "u8" => Ok(Self::U8(parse_with_type!(value, u8))),
            "u16" => Ok(Self::U16(parse_with_type!(value, u16))),
            "u32" => Ok(Self::U32(parse_with_type!(value, u32))),
            "u64" => Ok(Self::U64(parse_with_type!(value, u64))),
            "u128" => Ok(Self::U128(parse_with_type!(value, u128))),
            "u256" => {
                let num: BigUint = value.parse().context(single_value_parsing_error_msg(
                    value,
                    type_str_with_path,
                    None,
                ))?;
                let num_bytes = num.to_bytes_be();
                if num_bytes.len() > 32 {
                    bail!(single_value_parsing_error_msg(
                        value,
                        "u256",
                        Some(": number too large to fit in 32 bytes".to_string())
                    ))
                }

                let mut result = [0u8; 32];
                let start = 32 - num_bytes.len();
                result[start..].copy_from_slice(&num_bytes);

                Ok(Self::U256(CairoU256::from_bytes(&result)))
            }
            "i8" => Ok(Self::I8(parse_with_type!(value, i8))),
            "i16" => Ok(Self::I16(parse_with_type!(value, i16))),
            "i32" => Ok(Self::I32(parse_with_type!(value, i32))),
            "i64" => Ok(Self::I64(parse_with_type!(value, i64))),
            "i128" => Ok(Self::I128(parse_with_type!(value, i128))),
            // TODO check if bytes31 is actually a felt
            // (e.g. alexandria_data_structures::bit_array::BitArray uses that)
            // https://github.com/starkware-libs/cairo/blob/bf48e658b9946c2d5446eeb0c4f84868e0b193b5/corelib/src/bytes_31.cairo#L14
            // There is `bytes31_try_from_felt252`, which means it isn't always a valid felt?
            "felt252" | "felt" | "ContractAddress" | "ClassHash" | "bytes31" => {
                let felt = Felt::from_dec_str(value).context(single_value_parsing_error_msg(
                    value,
                    type_str_with_path,
                    None,
                ))?;
                Ok(Self::Felt(felt))
            }
            "bool" => Ok(Self::Bool(parse_with_type!(value, bool))),
            "ByteArray" => Ok(Self::ByteArray(ByteArray::from(value))),
            _ => {
                bail!(single_value_parsing_error_msg(
                    value,
                    type_str_with_path,
                    Some(format!(": unsupported type {type_str_with_path}"))
                ))
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct CalldataTuple(Vec<AllowedCalldataArguments>);

impl CalldataTuple {
    pub fn new(arguments: Vec<AllowedCalldataArguments>) -> Self {
        Self(arguments)
    }
}

#[derive(Debug)]
pub(crate) enum AllowedCalldataArguments {
    Struct(CalldataStruct),
    ArrayMacro(CalldataArrayMacro),
    Enum(CalldataEnum),
    // TODO rename to BasicType or smth
    SingleArgument(CalldataSingleArgument),
    Tuple(CalldataTuple),
}

impl CairoSerialize for CalldataSingleArgument {
    // https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/
    fn serialize(&self, output: &mut BufferWriter) {
        match self {
            CalldataSingleArgument::Felt(value) => value.serialize(output),
            CalldataSingleArgument::I8(value) => value.serialize(output),
            CalldataSingleArgument::I16(value) => value.serialize(output),
            CalldataSingleArgument::I32(value) => value.serialize(output),
            CalldataSingleArgument::I64(value) => value.serialize(output),
            CalldataSingleArgument::I128(value) => value.serialize(output),
            CalldataSingleArgument::U8(value) => value.serialize(output),
            CalldataSingleArgument::U16(value) => value.serialize(output),
            CalldataSingleArgument::U32(value) => value.serialize(output),
            CalldataSingleArgument::U64(value) => value.serialize(output),
            CalldataSingleArgument::U128(value) => value.serialize(output),
            CalldataSingleArgument::U256(value) => value.serialize(output),
            CalldataSingleArgument::Bool(value) => value.serialize(output),
            CalldataSingleArgument::ByteArray(value) => value.serialize(output),
        };
    }
}

impl CairoSerialize for CalldataStructField {
    // Every argument serialized in order of occurrence
    fn serialize(&self, output: &mut BufferWriter) {
        self.0.serialize(output);
    }
}

impl CairoSerialize for CalldataStruct {
    // https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/#serialization_of_structs
    fn serialize(&self, output: &mut BufferWriter) {
        self.0.iter().for_each(|field| field.serialize(output));
    }
}

impl CairoSerialize for CalldataTuple {
    fn serialize(&self, output: &mut BufferWriter) {
        self.0.iter().for_each(|field| field.serialize(output));
    }
}

impl CairoSerialize for CalldataArrayMacro {
    // https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/#serialization_of_arrays
    fn serialize(&self, output: &mut BufferWriter) {
        self.0.len().serialize(output);
        self.0.iter().for_each(|field| field.serialize(output));
    }
}

impl CairoSerialize for CalldataEnum {
    // https://docs.starknet.io/architecture-and-concepts/smart-contracts/serialization-of-cairo-types/#serialization_of_enums
    fn serialize(&self, output: &mut BufferWriter) {
        self.position.serialize(output);
        if self.argument.is_some() {
            self.argument.as_ref().unwrap().serialize(output);
        }
    }
}
impl CairoSerialize for AllowedCalldataArguments {
    fn serialize(&self, output: &mut BufferWriter) {
        match self {
            AllowedCalldataArguments::Struct(value) => value.serialize(output),
            AllowedCalldataArguments::ArrayMacro(value) => value.serialize(output),
            AllowedCalldataArguments::Enum(value) => value.serialize(output),
            AllowedCalldataArguments::SingleArgument(value) => value.serialize(output),
            AllowedCalldataArguments::Tuple(value) => value.serialize(output),
        }
    }
}

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

            let parsed_exprs = called_function
                .inputs
                .iter()
                .zip(arguments_expr_list)
                .map(|(param, arg)| parse_expr(arg, param.r#type.clone(), &abi, &db))
                .collect::<Result<Vec<AllowedCalldataArguments>>>()?;

            Ok(parsed_exprs
                .iter()
                .flat_map(SerializeToFeltVec::serialize_to_vec)
                .collect::<Vec<Felt>>())
        }
        ContractClass::Legacy(_legacy_class) => {
            bail!("Cairo-like expressions are not available for Cairo0 contracts")
        }
    }
}
