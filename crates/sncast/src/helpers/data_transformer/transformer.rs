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

#[cfg(test)]
mod tests {
    use super::transform_input_calldata;
    use conversions::string::TryFromHexStr;
    use conversions::IntoConv;
    use shared::rpc::create_rpc_client;
    use starknet::core::types::Felt;
    use starknet::core::utils::get_selector_from_name;

    // https://sepolia.starkscan.co/class/0x02a9b456118a86070a8c116c41b02e490f3dcc9db3cad945b4e9a7fd7cec9168#code
    const DATA_TRANSFORMER_TEST_CLASS_HASH: &str =
        "0x02a9b456118a86070a8c116c41b02e490f3dcc9db3cad945b4e9a7fd7cec9168";

    // 2^128 + 3
    const BIG_NUMBER: &str = "340282366920938463463374607431768211459";

    fn u128s_to_felts(vec: Vec<u128>) -> Vec<Felt> {
        vec.into_iter().map(Felt::from).collect()
    }

    #[tokio::test]
    async fn test_invalid_input() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            " 0x1 }",
            &get_selector_from_name("simple_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output
            .unwrap_err()
            .to_string()
            .contains("Couldn't parse input calldata, missing {"));

        //////////////

        let output = transform_input_calldata(
            "{ 0x1",
            &get_selector_from_name("simple_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output
            .unwrap_err()
            .to_string()
            .contains("Couldn't parse input calldata, missing }"));

        //////////////

        let output = transform_input_calldata(
            "0x1",
            &get_selector_from_name("simple_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output
            .unwrap_err()
            .to_string()
            .contains("Couldn't parse input calldata, missing {"));
    }
    #[tokio::test]
    async fn test_function_not_found() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let selector = get_selector_from_name("nonexistent_fn").unwrap();

        let output =
            transform_input_calldata("{ 0x1 }", &selector, contract_address, &client).await;

        assert!(output.is_err());
        assert!(output.unwrap_err().to_string().contains(
            format!(r#"Function with selector "{selector}" not found in ABI of the contract"#,)
                .as_str()
        ));
    }
    #[tokio::test]
    async fn test_invalid_suffix() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            "{ 1_u10 }",
            &get_selector_from_name("simple_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output
            .unwrap_err()
            .to_string()
            .contains(r#"Failed to parse value "1" into type "u10": unsupported type"#));
    }
    #[tokio::test]
    async fn test_number_suffix() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            "{ 1_u256 }",
            &get_selector_from_name("simple_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_ok());
        // TODO not sure about that behaviour, simple_fn accepts felt252
        let expected_output: Vec<Felt> = u128s_to_felts(vec![1, 0]);

        assert_eq!(output.unwrap(), expected_output);
    }
    #[tokio::test]
    async fn test_invalid_cairo_expression() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            "{ aaa: }",
            &get_selector_from_name("simple_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output
            .unwrap_err()
            .to_string()
            .contains("Invalid Cairo expression found in input calldata:"));
    }
    #[tokio::test]
    async fn test_invalid_arg_number() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            "{ 0x1, 0x2, 0x3 }",
            &get_selector_from_name("simple_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output
            .unwrap_err()
            .to_string()
            .contains("Invalid number of arguments, passed 3, expected 1"));
    }
    #[tokio::test]
    async fn test_simple_fn() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn simple_fn(self:@T, a: felt252);
            "{ 0x1 }",
            &get_selector_from_name("simple_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_ok());
        let expected_output: Vec<Felt> = u128s_to_felts(vec![0x1]);

        assert_eq!(output.unwrap(), expected_output);
    }

    #[tokio::test]
    async fn test_u256_fn() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn u256_fn(self:@T, a: u256);
            format!("{{ {BIG_NUMBER} }}").as_str(),
            &get_selector_from_name("u256_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_ok());
        let expected_output: Vec<Felt> = u128s_to_felts(vec![3, 1]);

        assert_eq!(output.unwrap(), expected_output);
    }
    #[tokio::test]
    async fn test_signed_fn() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn signed_fn(self:@T, a: i32);
            "{ -1 }",
            &get_selector_from_name("signed_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_ok());
        let expected_output: Vec<Felt> = vec![Felt::from(-1).into_()];

        assert_eq!(output.unwrap(), expected_output);
    }
    #[tokio::test]
    async fn test_signed_fn_overflow() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        // i32max = 2147483647
        let output = transform_input_calldata(
            // fn signed_fn(self:@T, a: i32);
            "{ 2147483648 }",
            &get_selector_from_name("signed_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output
            .unwrap_err()
            .to_string()
            .contains(r#"Failed to parse value "2147483648" into type "i32""#));
    }
    #[tokio::test]
    async fn test_unsigned_fn() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        // u32max = 4294967295
        let output = transform_input_calldata(
            // fn unsigned_fn(self:@T, a: u32);
            "{ 4294967295 }",
            &get_selector_from_name("unsigned_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_ok());
        let expected_output: Vec<Felt> = u128s_to_felts(vec![4_294_967_295]);

        assert_eq!(output.unwrap(), expected_output);
    }
    #[tokio::test]
    async fn test_tuple_fn() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn tuple_fn(self:@T, a: (felt252, u8, Enum));
            "{ (123, 234, Enum::Three(NestedStructWithField {a: SimpleStruct {a: 345}, b: 456 })) }",
            &get_selector_from_name("tuple_fn").unwrap(),
            contract_address,
            &client
        ).await;

        assert!(output.is_ok());
        let expected_output: Vec<Felt> = u128s_to_felts(vec![123, 234, 2, 345, 456]);

        assert_eq!(output.unwrap(), expected_output);
    }
    #[tokio::test]
    async fn test_complex_fn() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn complex_fn(self: @T, arr: Array<Array<felt252>>, one: u8, two: i16, three: ByteArray, four: (felt252, u32), five: bool, six: u256);
            r#"{ array![array![0,1,2], array![3,4,5,6,7]], 8, 9, "ten", (11, 12), true, 13 }"#,
            &get_selector_from_name("complex_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_ok());
        let expected_output: Vec<Felt> = u128s_to_felts(vec![
            2, 3, 0, 1, 2, 5, 3, 4, 5, 6, 7, 8, 9, 0, 7_628_142, 3, 11, 12, 1, 13, 0,
        ]);

        assert_eq!(output.unwrap(), expected_output);
    }
    #[tokio::test]
    async fn test_simple_struct_fn() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn simple_struct_fn(self: @T, a: SimpleStruct);
            "{ SimpleStruct {a: 0x12} }",
            &get_selector_from_name("simple_struct_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_ok());
        let expected_output: Vec<Felt> = u128s_to_felts(vec![0x12]);

        assert_eq!(output.unwrap(), expected_output);
    }

    #[tokio::test]
    async fn test_simple_struct_fn_invalid_struct_arg() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn simple_struct_fn(self: @T, a: SimpleStruct);
            r#"{ SimpleStruct {a: "string"} }"#,
            &get_selector_from_name("simple_struct_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output
            .unwrap_err()
            .to_string()
            .contains(r#"Failed to parse value "string" into type "core::felt252""#));
    }
    #[tokio::test]
    async fn test_simple_struct_fn_invalid_struct_name() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn simple_struct_fn(self: @T, a: SimpleStruct);
            r#"{ InvalidStructName {a: "string"} }"#,
            &get_selector_from_name("simple_struct_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output.unwrap_err().to_string().contains(r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got "InvalidStructName""#));
    }
    #[tokio::test]
    async fn test_simple_struct_fn_invalid_arg() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn simple_struct_fn(self: @T, a: SimpleStruct);
            "{ 0x1 }",
            &get_selector_from_name("simple_struct_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output.unwrap_err().to_string().contains(
            r#"Failed to parse value "1" into type "data_transformer_contract::SimpleStruct""#
        ));

        let output = transform_input_calldata(
            r#"{ "string_argument" }"#,
            &get_selector_from_name("simple_struct_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output.unwrap_err().to_string().contains(r#"Failed to parse value "string_argument" into type "data_transformer_contract::SimpleStruct""#));

        let output = transform_input_calldata(
            "{ 'shortstring' }",
            &get_selector_from_name("simple_struct_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output.unwrap_err().to_string().contains(r#"Failed to parse value "shortstring" into type "data_transformer_contract::SimpleStruct""#));

        let output = transform_input_calldata(
            "{ true }",
            &get_selector_from_name("simple_struct_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output.unwrap_err().to_string().contains(
            r#"Failed to parse value "true" into type "data_transformer_contract::SimpleStruct""#
        ));

        let output = transform_input_calldata(
            "{ array![0x1, 2, 0x3, 04] }",
            &get_selector_from_name("simple_struct_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output.unwrap_err().to_string().contains(
            r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got array"#
        ));

        let output = transform_input_calldata(
            "{ (1, array![2], 0x3) }",
            &get_selector_from_name("simple_struct_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output.unwrap_err().to_string().contains(
            r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got tuple"#
        ));

        let output = transform_input_calldata(
            "{ My::Enum }",
            &get_selector_from_name("simple_struct_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output.unwrap_err().to_string().contains(
            r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got "My""#
        ));

        let output = transform_input_calldata(
            "{ core::path::My::Enum(10) }",
            &get_selector_from_name("simple_struct_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output.unwrap_err().to_string().contains(r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got "core::path::My""#));
    }

    #[tokio::test]
    async fn test_nested_struct_fn() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn nested_struct_fn(self: @T, a: NestedStructWithField);
            "{ NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 } }",
            &get_selector_from_name("nested_struct_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_ok());

        let expected_output: Vec<Felt> = u128s_to_felts(vec![0x24, 96]);

        assert_eq!(output.unwrap(), expected_output);
    }

    // enum Enum
    // One,
    // #[default]
    // Two: u128,
    // Three: NestedStructWithField

    #[tokio::test]
    async fn test_enum_fn() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn enum_fn(self: @T, a: Enum);
            "{ Enum::One }",
            &get_selector_from_name("enum_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;
        assert!(output.is_ok());

        let expected_output: Vec<Felt> = u128s_to_felts(vec![0]);

        assert_eq!(output.unwrap(), expected_output);

        /////////////

        let output = transform_input_calldata(
            "{ Enum::Two(128) }",
            &get_selector_from_name("enum_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;
        assert!(output.is_ok());

        let expected_output: Vec<Felt> = u128s_to_felts(vec![1, 128]);

        assert_eq!(output.unwrap(), expected_output);

        /////////////

        let output = transform_input_calldata(
            "{ Enum::Three(NestedStructWithField { a: SimpleStruct { a: 123 }, b: 234 }) }",
            &get_selector_from_name("enum_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;
        assert!(output.is_ok());

        let expected_output: Vec<Felt> = u128s_to_felts(vec![2, 123, 234]);

        assert_eq!(output.unwrap(), expected_output);
    }

    #[tokio::test]
    async fn test_enum_fn_invalid_variant() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn enum_fn(self: @T, a: Enum);
            "{ Enum::Four }",
            &get_selector_from_name("enum_fn").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_err());
        assert!(output
            .unwrap_err()
            .to_string()
            .contains(r#"Couldn't find variant "Four" in enum "Enum""#));
    }

    #[tokio::test]
    async fn test_complex_struct_fn() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        // struct ComplexStruct
        //     a: NestedStructWithField,
        //     b: felt252,
        //     c: u8,
        //     d: i32,
        //     e: Enum,
        //     f: ByteArray,
        //     g: Array<felt252>,
        //     h: u256,
        //     i: (i128, u128),

        let output = transform_input_calldata(
            // fn complex_struct_fn(self: @T, a: ComplexStruct);
            r#"{ ComplexStruct {a: NestedStructWithField { a: SimpleStruct { a: 1 }, b: 2 }, b: 3, c: 4, d: 5, e: Enum::Two(6), f: "seven", g: array![8, 9], h: 10, i: (11, 12) } }"#,
            &get_selector_from_name("complex_struct_fn").unwrap(),
            contract_address,
            &client
        ).await;
        assert!(output.is_ok());

        // 1 2 - a: NestedStruct
        // 3 - b: felt252
        // 4 - c: u8
        // 5 - d: i32
        // 1 6 - e: Enum
        // 0 495623497070 5 - f: string (ByteArray)
        // 2 8 9 - g: array!
        // 10 0 - h: u256
        // 11 12 - i: (i128, u128)
        let expected_output: Vec<Felt> = u128s_to_felts(vec![
            1,
            2,
            3,
            4,
            5,
            1,
            6,
            0,
            495_623_497_070,
            5,
            2,
            8,
            9,
            10,
            0,
            11,
            12,
        ]);

        assert_eq!(output.unwrap(), expected_output);
    }

    // TODO add similar test but with enums
    //  - take existing contract code
    //  - find/create a library with an enum
    //  - add to project as a dependency
    //  - create enum with the same name in your contract code
    #[tokio::test]
    async fn test_external_struct_fn_ambiguity() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn external_struct_fn(self:@T, a: BitArray, b: bit_array::BitArray);
            "{ BitArray { bit: 23 }, BitArray { data: array![0], current: 1, read_pos: 2, write_pos: 3 } }",
            &get_selector_from_name("external_struct_fn").unwrap(),
            contract_address,
            &client
        ).await;

        assert!(output.is_err());
        assert!(output.unwrap_err().to_string().contains(r#"Found more than one struct "BitArray" in ABI, please specify a full path to the struct"#));
    }

    #[tokio::test]
    async fn test_external_struct_fn_invalid_path() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn external_struct_fn(self:@T, a: BitArray, b: bit_array::BitArray);
            "{ something::BitArray { bit: 23 }, BitArray { data: array![0], current: 1, read_pos: 2, write_pos: 3 } }",
            &get_selector_from_name("external_struct_fn").unwrap(),
            contract_address,
            &client
        ).await;

        assert!(output.is_err());
        assert!(output
            .unwrap_err()
            .to_string()
            .contains(r#"Struct "something::BitArray" not found in ABI"#));
    }
    #[tokio::test]
    async fn test_external_struct_fn() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn external_struct_fn(self:@T, a: BitArray, b: bit_array::BitArray);
            "{ data_transformer_contract::BitArray { bit: 23 }, alexandria_data_structures::bit_array::BitArray { data: array![0], current: 1, read_pos: 2, write_pos: 3 } }",
            &get_selector_from_name("external_struct_fn").unwrap(),
            contract_address,
            &client
        ).await;

        assert!(output.is_ok());

        let expected_output: Vec<Felt> = u128s_to_felts(vec![23, 1, 0, 1, 2, 3]);

        assert_eq!(output.unwrap(), expected_output);
    }
    #[tokio::test]
    async fn test_constructor() {
        let client = create_rpc_client("http://188.34.188.184:7070/rpc/v0_7").unwrap();
        let contract_address: Felt =
            Felt::try_from_hex_str(DATA_TRANSFORMER_TEST_CLASS_HASH).unwrap();

        let output = transform_input_calldata(
            // fn constructor(ref self: ContractState, init_owner: ContractAddress) {}
            "{ 0x123 }",
            &get_selector_from_name("constructor").unwrap(),
            contract_address,
            &client,
        )
        .await;

        assert!(output.is_ok());

        let expected_output: Vec<Felt> = u128s_to_felts(vec![0x123]);

        assert_eq!(output.unwrap(), expected_output);
    }
}
