use itertools::Itertools;
use primitive_types::U256;
use shared::rpc::create_rpc_client;
use sncast::helpers::data_transformer::transformer::transform;
use starknet::core::types::{BlockId, BlockTag, ContractClass, Felt};
use starknet::core::utils::get_selector_from_name;
use starknet::providers::Provider;
use tokio::sync::OnceCell;

const RPC_ENDPOINT: &str = "http://188.34.188.184:7070/rpc/v0_7";

// https://sepolia.starkscan.co/class/0x02a9b456118a86070a8c116c41b02e490f3dcc9db3cad945b4e9a7fd7cec9168#code
const TEST_CLASS_HASH: Felt =
    Felt::from_hex_unchecked("0x02a9b456118a86070a8c116c41b02e490f3dcc9db3cad945b4e9a7fd7cec9168");

static CLASS: OnceCell<ContractClass> = OnceCell::const_new();

// 2^128 + 3
// const BIG_NUMBER: &str = "340282366920938463463374607431768211459";

async fn init_class() -> ContractClass {
    let client = create_rpc_client(RPC_ENDPOINT).unwrap();

    client
        .get_class(BlockId::Tag(BlockTag::Latest), TEST_CLASS_HASH)
        .await
        .unwrap()
}

// #[tokio::test]
// async fn test_happy_case_simple_function_with_maunally_serialized_input() -> anyhow::Result<()> {
//     let serialized_calldata: Vec<Felt> = vec![100.into()];
//     let simulated_cli_input: Vec<FeltOrString> = serialized_calldata
//         .clone()
//         .into_iter()
//         .map(From::from)
//         .collect();

//     let contract_class = CLASS.get_or_init(init_class).await.to_owned();

//     let result = transform(
//         simulated_cli_input,
//         contract_class,
//         &get_selector_from_name("simple_fn").unwrap(),
//     )
//     .await?;

//     assert_eq!(result, serialized_calldata);

//     Ok(())
// }

#[tokio::test]
async fn test_happy_case_tuple_function() -> anyhow::Result<()> {
    let simulated_cli_input = vec![String::from("(2137_felt252, 1_u8, Enum::One)")];

    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    transform(
        &simulated_cli_input,
        contract_class,
        &get_selector_from_name("tuple_fn").unwrap(),
    )?;

    Ok(())
}

#[tokio::test]
async fn test_happy_case_complex_function_cairo_expressions_input_only() -> anyhow::Result<()> {
    let max_u256 = U256::max_value().to_string();

    let simulated_cli_input = vec![
        "array![array![0x2137, 0x420], array![0x420, 0x2137]]",
        "8_u8",
        "-270",
        "\"some string\"",
        "(0x69, 100)",
        "true",
        &max_u256,
    ]
    .into_iter()
    .map(String::from)
    .collect_vec();

    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    transform(
        &simulated_cli_input,
        contract_class,
        &get_selector_from_name("complex_fn").unwrap(),
    )?;

    Ok(())
}

#[allow(unreachable_code, unused_variables, clippy::diverging_sub_expression)]
#[ignore = "Prepare serialized data by-hand"]
#[tokio::test]
async fn test_happy_case_complex_function_serialized_input_only() -> anyhow::Result<()> {
    let simulated_cli_input: Vec<String> = todo!();

    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    transform(
        &simulated_cli_input,
        contract_class,
        &get_selector_from_name("complex_fn").unwrap(),
    )?;

    Ok(())
}

#[allow(unreachable_code, unused_variables, clippy::diverging_sub_expression)]
#[ignore = "Prepare serialized data by-hand"]
#[tokio::test]
async fn test_happy_case_complex_function_mixed_input() -> anyhow::Result<()> {
    let simulated_cli_input: Vec<String> = todo!();

    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    transform(
        &simulated_cli_input,
        contract_class,
        &get_selector_from_name("complex_fn").unwrap(),
    )?;

    Ok(())
}

#[tokio::test]
async fn test_function_not_found() {
    let simulated_cli_input = vec![String::from("'some_felt'")];

    let contract_class = CLASS.get_or_init(init_class).await.to_owned();
    let selector = get_selector_from_name("nonexistent_fn").unwrap();

    let output = transform(&simulated_cli_input, contract_class, &selector);

    assert!(output.is_err());
    assert!(output.unwrap_err().to_string().contains(
        format!(r#"Function with selector "{selector}" not found in ABI of the contract"#,)
            .as_str()
    ));
}

#[tokio::test]
async fn test_happy_case_numeric_type_suffix() -> anyhow::Result<()> {
    let simulated_cli_input = vec![String::from("1010101_u32")];

    let contract_class = CLASS.get_or_init(init_class).await.to_owned();
    let selector = get_selector_from_name("unsigned_fn").unwrap();

    let output = transform(&simulated_cli_input, contract_class, &selector)?;

    assert_eq!(output, vec![Felt::from(1_010_101_u32)]);

    Ok(())
}

#[tokio::test]
async fn test_invalid_numeric_type_suffix() {
    let simulated_cli_input = vec![String::from("1_u10")];

    let contract_class = CLASS.get_or_init(init_class).await.to_owned();
    let selector = get_selector_from_name("simple_fn").unwrap();

    let output = transform(&simulated_cli_input, contract_class, &selector);

    assert!(output.is_err());
    assert!(output
        .unwrap_err()
        .to_string()
        .contains(r#"Failed to parse value "1" into type "u10": unsupported type"#));
}

#[tokio::test]
async fn test_invalid_cairo_expression() {
    let simulated_cli_input = vec![String::from("some_invalid_expression:")];

    let contract_class = CLASS.get_or_init(init_class).await.to_owned();
    let selector = get_selector_from_name("simple_fn").unwrap();

    let output = transform(&simulated_cli_input, contract_class, &selector);

    assert!(output.is_err());
    assert!(output
        .unwrap_err()
        .to_string()
        .contains("Invalid Cairo expression found in input calldata"));
}

#[tokio::test]
async fn test_invalid_argument_number() {
    let simulated_cli_input = vec!["0x123", "'some_obsolete_argument'", "10"]
        .into_iter()
        .map(String::from)
        .collect_vec();

    let contract_class = CLASS.get_or_init(init_class).await.to_owned();
    let selector = get_selector_from_name("simple_fn").unwrap();

    let output = transform(&simulated_cli_input, contract_class, &selector);

    assert!(output.is_err());
    assert!(output
        .unwrap_err()
        .to_string()
        .contains("Invalid number of arguments, passed 3, expected 1"));
}

// #[tokio::test]
// async fn test_happy_case_u256_fn() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn u256_fn(self: @T, a: u256);
//         format!("{{ {BIG_NUMBER} }}").as_str(),
//         &get_selector_from_name("u256_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client,
//     )
//     .await;

//     assert!(output.is_ok());
//     let expected_output: Vec<Felt> = to_felt_vector(vec![3, 1]);

//     assert_eq!(output.unwrap(), expected_output);
// }

// #[tokio::test]
// async fn test_happy_case_signed_fn() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn signed_fn(self: @T, a: i32);
//         "{ -1 }",
//         &get_selector_from_name("signed_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client,
//     )
//     .await;

//     assert!(output.is_ok());
//     let expected_output: Vec<Felt> = vec![Felt::from(-1).into_()];

//     assert_eq!(output.unwrap(), expected_output);
// }

#[tokio::test]
async fn test_signed_fn_overflow() {
    let simulated_cli_input = vec![(i32::MAX as u64 + 1).to_string()];

    let contract_class = CLASS.get_or_init(init_class).await.to_owned();
    let selector = get_selector_from_name("signed_fn").unwrap();

    let output = transform(&simulated_cli_input, contract_class, &selector);

    assert!(output.is_err());
    assert!(output
        .unwrap_err()
        .to_string()
        .contains(r#"Failed to parse value "2147483648" into type "i32""#));
}

// #[tokio::test]
// async fn test_happy_case_unsigned_fn() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     // u32max = 4294967295
//     let output = transform(
//         // fn unsigned_fn(self: @T, a: u32);
//         "{ 4294967295 }",
//         &get_selector_from_name("unsigned_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client,
//     )
//     .await;

//     assert!(output.is_ok());
//     let expected_output: Vec<Felt> = to_felt_vector(vec![4_294_967_295]);

//     assert_eq!(output.unwrap(), expected_output);
// }

// #[tokio::test]
// async fn test_happy_case_tuple_fn() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn tuple_fn(self: @T, a: (felt252, u8, Enum));
//         "{ (123, 234, Enum::Three(NestedStructWithField {a: SimpleStruct {a: 345}, b: 456 })) }",
//         &get_selector_from_name("tuple_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client,
//     )
//     .await;

//     assert!(output.is_ok());
//     let expected_output: Vec<Felt> = to_felt_vector(vec![123, 234, 2, 345, 456]);

//     assert_eq!(output.unwrap(), expected_output);
// }

// #[tokio::test]
// async fn test_happy_case_complex_fn() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn complex_fn(self: @T, arr: Array<Array<felt252>>, one: u8, two: i16, three: ByteArray, four: (felt252, u32), five: bool, six: u256);
//         r#"{ array![array![0,1,2], array![3,4,5,6,7]], 8, 9, "ten", (11, 12), true, 13 }"#,
//         &get_selector_from_name("complex_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client,
//     )
//     .await;

//     assert!(output.is_ok());
//     let expected_output: Vec<Felt> = to_felt_vector(vec![
//         2, 3, 0, 1, 2, 5, 3, 4, 5, 6, 7, 8, 9, 0, 7_628_142, 3, 11, 12, 1, 13, 0,
//     ]);

//     assert_eq!(output.unwrap(), expected_output);
// }

// #[tokio::test]
// async fn test_happy_case_simple_struct_fn() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn simple_struct_fn(self: @T, a: SimpleStruct);
//         "{ SimpleStruct {a: 0x12} }",
//         &get_selector_from_name("simple_struct_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client,
//     )
//     .await;

//     assert!(output.is_ok());
//     let expected_output: Vec<Felt> = to_felt_vector(vec![0x12]);

//     assert_eq!(output.unwrap(), expected_output);
// }

// #[tokio::test]
// async fn test_simple_struct_fn_invalid_struct_argument() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn simple_struct_fn(self: @T, a: SimpleStruct);
//         r#"{ SimpleStruct {a: "string"} }"#,
//         &get_selector_from_name("simple_struct_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client,
//     )
//     .await;

//     assert!(output.is_err());
//     assert!(output
//         .unwrap_err()
//         .to_string()
//         .contains(r#"Failed to parse value "string" into type "core::felt252""#));
// }

// #[tokio::test]
// async fn test_simple_struct_fn_invalid_struct_name() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn simple_struct_fn(self: @T, a: SimpleStruct);
//         r#"{ InvalidStructName {a: "string"} }"#,
//         &get_selector_from_name("simple_struct_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client,
//     )
//     .await;

//     assert!(output.is_err());
//     assert!(output.unwrap_err().to_string().contains(r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got "InvalidStructName""#));
// }

// #[test_case("{ 0x1 }", r#"Failed to parse value "1" into type "data_transformer_contract::SimpleStruct""# ; "felt")]
// #[test_case(r#"{ "string_argument" }"#, r#"Failed to parse value "string_argument" into type "data_transformer_contract::SimpleStruct""# ; "string")]
// #[test_case("{ 'shortstring' }", r#"Failed to parse value "shortstring" into type "data_transformer_contract::SimpleStruct""# ; "shortstring")]
// #[test_case("{ true }", r#"Failed to parse value "true" into type "data_transformer_contract::SimpleStruct""# ; "bool")]
// #[test_case("{ array![0x1, 2, 0x3, 04] }", r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got array"# ; "array")]
// #[test_case("{ (1, array![2], 0x3) }", r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got tuple"# ; "tuple")]
// #[test_case("{ My::Enum }", r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got "My""# ; "enum_variant")]
// #[test_case("{ core::path::My::Enum(10) }", r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got "core::path::My""# ; "enum_variant_with_path")]
// #[tokio::test]
// async fn test_simple_struct_fn_invalid_argument(input: &str, error_message: &str) {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn simple_struct_fn(self: @T, a: SimpleStruct);
//         input,
//         &get_selector_from_name("simple_struct_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client,
//     )
//     .await;

//     assert!(output.is_err());
//     assert!(output.unwrap_err().to_string().contains(error_message));
// }

// #[tokio::test]
// async fn test_happy_case_nested_struct_fn() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn nested_struct_fn(self: @T, a: NestedStructWithField);
//         "{ NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 } }",
//         &get_selector_from_name("nested_struct_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client,
//     )
//     .await;

//     assert!(output.is_ok());

//     let expected_output: Vec<Felt> = to_felt_vector(vec![0x24, 96]);

//     assert_eq!(output.unwrap(), expected_output);
// }

// // enum Enum
// // One,
// // #[default]
// // Two: u128,
// // Three: NestedStructWithField
// //
// #[test_case("{ Enum::One }", to_felt_vector(vec![0]) ; "empty_variant")]
// #[test_case("{ Enum::Two(128) }", to_felt_vector(vec![1, 128]) ; "one_argument_variant")]
// #[test_case(
//     "{ Enum::Three(NestedStructWithField { a: SimpleStruct { a: 123 }, b: 234 }) }",
//     to_felt_vector(vec![2, 123, 234]);
//     "nested_struct_variant"
// )]
// #[tokio::test]
// async fn test_happy_case_enum_fn(input: &str, expected_output: Vec<Felt>) {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn enum_fn(self: @T, a: Enum);
//         input,
//         &get_selector_from_name("enum_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client,
//     )
//     .await;

//     assert!(output.is_ok());
//     assert_eq!(output.unwrap(), expected_output);
// }

// #[tokio::test]
// async fn test_happy_case_enum_fn_invalid_variant() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn enum_fn(self: @T, a: Enum);
//         "{ Enum::Four }",
//         &get_selector_from_name("enum_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client,
//     )
//     .await;

//     assert!(output.is_err());
//     assert!(output
//         .unwrap_err()
//         .to_string()
//         .contains(r#"Couldn't find variant "Four" in enum "Enum""#));
// }

// #[tokio::test]
// async fn test_happy_case_complex_struct_fn() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     // struct ComplexStruct
//     //     a: NestedStructWithField,
//     //     b: felt252,
//     //     c: u8,
//     //     d: i32,
//     //     e: Enum,
//     //     f: ByteArray,
//     //     g: Array<felt252>,
//     //     h: u256,
//     //     i: (i128, u128),

//     let output = transform(
//         // fn complex_struct_fn(self: @T, a: ComplexStruct);
//         r#"{ ComplexStruct {a: NestedStructWithField { a: SimpleStruct { a: 1 }, b: 2 }, b: 3, c: 4, d: 5, e: Enum::Two(6), f: "seven", g: array![8, 9], h: 10, i: (11, 12) } }"#,
//         &get_selector_from_name("complex_struct_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client
//     ).await;
//     assert!(output.is_ok());

//     // 1 2 - a: NestedStruct
//     // 3 - b: felt252
//     // 4 - c: u8
//     // 5 - d: i32
//     // 1 6 - e: Enum
//     // 0 495623497070 5 - f: string (ByteArray)
//     // 2 8 9 - g: array!
//     // 10 0 - h: u256
//     // 11 12 - i: (i128, u128)
//     let expected_output: Vec<Felt> = to_felt_vector(vec![
//         1,
//         2,
//         3,
//         4,
//         5,
//         1,
//         6,
//         0,
//         495_623_497_070,
//         5,
//         2,
//         8,
//         9,
//         10,
//         0,
//         11,
//         12,
//     ]);

//     assert_eq!(output.unwrap(), expected_output);
// }

// // TODO add similar test but with enums
// //  - take existing contract code
// //  - find/create a library with an enum
// //  - add to project as a dependency
// //  - create enum with the same name in your contract code
// #[tokio::test]
// async fn test_ambiguous_struct() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn external_struct_fn(self:@T, a: BitArray, b: bit_array::BitArray);
//         "{ BitArray { bit: 23 }, BitArray { data: array![0], current: 1, read_pos: 2, write_pos: 3 } }",
//         &get_selector_from_name("external_struct_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client
//     ).await;

//     assert!(output.is_err());
//     assert!(output.unwrap_err().to_string().contains(
//         r#"Found more than one struct "BitArray" in ABI, please specify a full path to the struct"#
//     ));
// }

// #[tokio::test]
// async fn test_invalid_path_to_external_struct() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn external_struct_fn(self:@T, a: BitArray, b: bit_array::BitArray);
//         "{ something::BitArray { bit: 23 }, BitArray { data: array![0], current: 1, read_pos: 2, write_pos: 3 } }",
//         &get_selector_from_name("external_struct_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client
//     ).await;

//     assert!(output.is_err());
//     assert!(output
//         .unwrap_err()
//         .to_string()
//         .contains(r#"Struct "something::BitArray" not found in ABI"#));
// }

// #[tokio::test]
// async fn test_happy_case_path_to_external_struct() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn external_struct_fn(self:@T, a: BitArray, b: bit_array::BitArray);
//         "{ data_transformer_contract::BitArray { bit: 23 }, alexandria_data_structures::bit_array::BitArray { data: array![0], current: 1, read_pos: 2, write_pos: 3 } }",
//         &get_selector_from_name("external_struct_fn").unwrap(),
//         TEST_CLASS_HASH,
//         &client
//     ).await;

//     assert!(output.is_ok());

//     let expected_output: Vec<Felt> = to_felt_vector(vec![23, 1, 0, 1, 2, 3]);

//     assert_eq!(output.unwrap(), expected_output);
// }

// #[tokio::test]
// async fn test_happy_case_contract_constructor() {
//     let client = create_rpc_client(RPC_ENDPOINT).unwrap();

//     let output = transform(
//         // fn constructor(ref self: ContractState, init_owner: ContractAddress) {}
//         "{ 0x123 }",
//         &get_selector_from_name("constructor").unwrap(),
//         TEST_CLASS_HASH,
//         &client,
//     )
//     .await;

//     assert!(output.is_ok());

//     let expected_output: Vec<Felt> = to_felt_vector(vec![0x123]);

//     assert_eq!(output.unwrap(), expected_output);
// }
