use core::fmt;
use data_transformer::Calldata;
use indoc::indoc;
use itertools::Itertools;
use primitive_types::U256;
use starknet::core::types::{BlockId, BlockTag, ContractClass};
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet_types_core::felt::Felt;
use std::ops::Not;
use test_case::test_case;
use tokio::sync::OnceCell;
use url::Url;

// https://sepolia.starkscan.co/class/0x02a9b456118a86070a8c116c41b02e490f3dcc9db3cad945b4e9a7fd7cec9168#code
const TEST_CLASS_HASH: Felt =
    Felt::from_hex_unchecked("0x032e6763d5e778f153e5b6ea44200d94ec89aac7b42a0aef0e4e0caac8dafdab");

static CLASS: OnceCell<ContractClass> = OnceCell::const_new();

async fn init_class() -> ContractClass {
    let client = JsonRpcClient::new(HttpTransport::new(
        Url::parse("http://188.34.188.184:7070/rpc/v0_8").unwrap(),
    ));

    client
        .get_class(BlockId::Tag(BlockTag::Latest), TEST_CLASS_HASH)
        .await
        .unwrap()
}

trait Contains<T: fmt::Debug + Eq> {
    fn assert_contains(&self, value: T);
}

impl Contains<&str> for anyhow::Error {
    fn assert_contains(&self, value: &str) {
        self.chain()
            .any(|err| err.to_string().contains(value))
            .not()
            .then(|| panic!("{value:?}\nnot found in\n{self:#?}"));
    }
}

#[tokio::test]
async fn test_function_not_found() {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();
    let selector = get_selector_from_name("nonexistent_fn").unwrap();

    let input = String::from("('some_felt',)");

    let result = Calldata::new(input).serialized(contract_class, &selector);

    result.unwrap_err().assert_contains(
        format!(r#"Function with selector "{selector}" not found in ABI of the contract"#).as_str(),
    );
}

#[tokio::test]
async fn test_happy_case_numeric_type_suffix() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("1010101_u32");

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("unsigned_fn").unwrap(),
    )?;

    let expected_output = [Felt::from(1_010_101_u32)];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_invalid_numeric_type_suffix() {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("1_u10");

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("simple_fn").unwrap(),
    );

    result
        .unwrap_err()
        .assert_contains(r#"Failed to parse value "1" into type "u10": unsupported type u10"#);
}

#[tokio::test]
async fn test_invalid_cairo_expression() {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("(some_invalid_expression:,)");

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("simple_fn").unwrap(),
    );

    result
        .unwrap_err()
        .assert_contains("Invalid Cairo expression found in input calldata");
}

#[tokio::test]
async fn test_invalid_argument_number() {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("0x123, 'some_obsolete_argument', 10");
    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("simple_fn").unwrap(),
    );

    result
        .unwrap_err()
        .assert_contains("Invalid number of arguments: passed 3, expected 1");
}

#[tokio::test]
async fn test_happy_case_simple_cairo_expressions_input() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("100");

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("simple_fn").unwrap(),
    )?;

    let expected_output = [Felt::from_hex_unchecked("0x64")];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_happy_case_u256_function_cairo_expressions_input_decimal() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = format!("{}_u256", U256::MAX);

    let result = Calldata::new(input)
        .serialized(contract_class, &get_selector_from_name("u256_fn").unwrap())?;

    let expected_output = [
        Felt::from_hex_unchecked("0xffffffffffffffffffffffffffffffff"),
        Felt::from_hex_unchecked("0xffffffffffffffffffffffffffffffff"),
    ];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_happy_case_u256_function_cairo_expressions_input_hex() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("0x2137_u256");

    let result = Calldata::new(input)
        .serialized(contract_class, &get_selector_from_name("u256_fn").unwrap())?;

    let expected_output = [
        Felt::from_hex_unchecked("0x2137"),
        Felt::from_hex_unchecked("0x0"),
    ];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_happy_case_signed_function_cairo_expressions_input() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("-273");

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("signed_fn").unwrap(),
    )?;

    let expected_output = [Felt::from(-273i16)];

    assert_eq!(result, expected_output);

    Ok(())
}

// Problem: Although transformer fails to process the given input as `i32`, it then succeeds to interpret it as `felt252`
// Overflow checks will not work for functions having the same serialized and Cairo-like calldata length.
// User must provide a type suffix or get the invoke-time error
// Issue #2559
#[ignore = "Impossible to pass with the current solution"]
#[tokio::test]
async fn test_signed_fn_overflow() {
    let input = format!("({},)", i32::MAX as u64 + 1);

    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("signed_fn").unwrap(),
    );

    result
        .unwrap_err()
        .assert_contains(r#"Failed to parse value "2147483648" into type "i32""#);
}

#[tokio::test]
async fn test_signed_fn_overflow_with_type_suffix() {
    let input = format!("{}_i32", i32::MAX as u64 + 1);

    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("signed_fn").unwrap(),
    );

    result
        .unwrap_err()
        .assert_contains(r#"Failed to parse value "2147483648" into type "i32""#);
}

#[tokio::test]
async fn test_happy_case_unsigned_function_cairo_expressions_input() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = format!("{}", u32::MAX);

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("unsigned_fn").unwrap(),
    )?;

    let expected_output = [Felt::from(u32::MAX)];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_happy_case_tuple_function_cairo_expression_input() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("(2137_felt252, 1_u8, Enum::One)");

    let result = Calldata::new(input)
        .serialized(contract_class, &get_selector_from_name("tuple_fn").unwrap())?;

    let expected_output = [
        Felt::from_hex_unchecked("0x859"),
        Felt::from_hex_unchecked("0x1"),
        Felt::from_hex_unchecked("0x0"),
    ];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_happy_case_tuple_function_with_nested_struct_cairo_expression_input()
-> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from(
        "(123, 234, Enum::Three(NestedStructWithField {a: SimpleStruct {a: 345}, b: 456 }))",
    );

    let result = Calldata::new(input)
        .serialized(contract_class, &get_selector_from_name("tuple_fn").unwrap())?;

    let expected_output = [123, 234, 2, 345, 456]
        .into_iter()
        .map(Felt::from)
        .collect_vec();

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_happy_case_complex_function_cairo_expressions_input() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let max_u256 = U256::max_value().to_string();

    let input = format!(
        "array![array![0x2137, 0x420], array![0x420, 0x2137]],
        8_u8,
        -270,
        \"some_string\",
        ('short string', 100),
        true,
        {max_u256}",
    );
    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("complex_fn").unwrap(),
    )?;

    // Manually serialized in Cairo
    let expected_output = [
        "0x2",
        "0x2",
        "0x2137",
        "0x420",
        "0x2",
        "0x420",
        "0x2137",
        "0x8",
        "0x800000000000010fffffffffffffffffffffffffffffffffffffffffffffef3",
        "0x0",
        "0x736f6d655f737472696e67",
        "0xb",
        "0x73686f727420737472696e67",
        "0x64",
        "0x1",
        "0xffffffffffffffffffffffffffffffff",
        "0xffffffffffffffffffffffffffffffff",
    ]
    .into_iter()
    .map(Felt::from_hex_unchecked)
    .collect_vec();

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_happy_case_simple_struct_function_cairo_expression_input() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("SimpleStruct {a: 0x12}");

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("simple_struct_fn").unwrap(),
    )?;

    let expected_output = [Felt::from_hex_unchecked("0x12")];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_simple_struct_function_invalid_struct_argument() {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from(r#"SimpleStruct {a: "string"}"#);

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("simple_struct_fn").unwrap(),
    );

    result
        .unwrap_err()
        .assert_contains(r#"Failed to parse value "string" into type "core::felt252""#);
}

#[tokio::test]
async fn test_simple_struct_function_invalid_struct_name() {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("InvalidStructName {a: 0x10}");

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("simple_struct_fn").unwrap(),
    );

    result
        .unwrap_err()
        .assert_contains(r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got "InvalidStructName""#);
}

#[test_case(r#""string_argument""#, r#"Failed to parse value "string_argument" into type "data_transformer_contract::SimpleStruct""# ; "string")]
#[test_case("'shortstring'", r#"Failed to parse value "shortstring" into type "data_transformer_contract::SimpleStruct""# ; "shortstring")]
#[test_case("true", r#"Failed to parse value "true" into type "data_transformer_contract::SimpleStruct""# ; "bool")]
#[test_case("array![0x1, 2, 0x3, 04]", r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got array"# ; "array")]
#[test_case("(1, array![2], 0x3)", r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got tuple"# ; "tuple")]
#[test_case("My::Enum", r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got "My""# ; "enum_variant")]
#[test_case("core::path::My::Enum(10)", r#"Invalid argument type, expected "data_transformer_contract::SimpleStruct", got "core::path::My""# ; "enum_variant_with_path")]
#[tokio::test]
async fn test_simple_struct_function_cairo_expression_input_invalid_argument_type(
    input: &str,
    error_message: &str,
) {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let result = Calldata::new(input.to_string()).serialized(
        contract_class,
        &get_selector_from_name("simple_struct_fn").unwrap(),
    );

    result.unwrap_err().assert_contains(error_message);
}

#[tokio::test]
async fn test_happy_case_nested_struct_function_cairo_expression_input() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }");

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("nested_struct_fn").unwrap(),
    )?;

    let expected_output = [
        Felt::from_hex_unchecked("0x24"),
        Felt::from_hex_unchecked("0x60"),
    ];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_happy_case_span_function_cairo_expression_input() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("array![1, 2, 3]");

    let result = Calldata::new(input)
        .serialized(contract_class, &get_selector_from_name("span_fn").unwrap())?;

    let expected_output = [
        Felt::from_hex_unchecked("0x3"),
        Felt::from_hex_unchecked("0x1"),
        Felt::from_hex_unchecked("0x2"),
        Felt::from_hex_unchecked("0x3"),
    ];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_happy_case_empty_span_function_cairo_expression_input() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("array![]");

    let result = Calldata::new(input)
        .serialized(contract_class, &get_selector_from_name("span_fn").unwrap())?;

    let expected_output = [Felt::from_hex_unchecked("0x0")];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_happy_case_enum_function_empty_variant_cairo_expression_input() -> anyhow::Result<()>
{
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("Enum::One");

    let result = Calldata::new(input)
        .serialized(contract_class, &get_selector_from_name("enum_fn").unwrap())?;

    let expected_output = [Felt::ZERO];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_happy_case_enum_function_one_argument_variant_cairo_expression_input()
-> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("Enum::Two(128)");

    let result = Calldata::new(input)
        .serialized(contract_class, &get_selector_from_name("enum_fn").unwrap())?;

    let expected_output = [
        Felt::from_hex_unchecked("0x1"),
        Felt::from_hex_unchecked("0x80"),
    ];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_happy_case_enum_function_nested_struct_variant_cairo_expression_input()
-> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input =
        String::from("Enum::Three(NestedStructWithField { a: SimpleStruct { a: 123 }, b: 234 })");

    let result = Calldata::new(input)
        .serialized(contract_class, &get_selector_from_name("enum_fn").unwrap())?;

    let expected_output = [
        Felt::from_hex_unchecked("0x2"),
        Felt::from_hex_unchecked("0x7b"),
        Felt::from_hex_unchecked("0xea"),
    ];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_enum_function_invalid_variant_cairo_expression_input() {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("Enum::InvalidVariant");

    let result = Calldata::new(input)
        .serialized(contract_class, &get_selector_from_name("enum_fn").unwrap());

    result
        .unwrap_err()
        .assert_contains(r#"Couldn't find variant "InvalidVariant" in enum "Enum""#);
}

#[tokio::test]
async fn test_happy_case_complex_struct_function_cairo_expression_input() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let data = indoc!(
        r#"
        ComplexStruct {
            a: NestedStructWithField {
                a: SimpleStruct { a: 1 },
                b: 2
            },
            b: 3, c: 4, d: 5,
            e: Enum::Two(6),
            f: "seven",
            g: array![8, 9],
            h: 10,
            i: (11, 12)
        }
        "#
    );

    let input = String::from(data);

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("complex_struct_fn").unwrap(),
    )?;

    let expected_output = [
        // a: NestedStruct
        Felt::from_hex_unchecked("0x1"),
        Felt::from_hex_unchecked("0x2"),
        // b: felt252
        Felt::from_hex_unchecked("0x3"),
        // c: u8
        Felt::from_hex_unchecked("0x4"),
        // d: i32
        Felt::from_hex_unchecked("0x5"),
        // e: Enum
        Felt::from_hex_unchecked("0x1"),
        Felt::from_hex_unchecked("0x6"),
        // f: ByteArray
        Felt::from_hex_unchecked("0x0"),
        Felt::from_hex_unchecked("0x736576656e"),
        Felt::from_hex_unchecked("0x5"),
        // g: Array
        Felt::from_hex_unchecked("0x2"),
        Felt::from_hex_unchecked("0x8"),
        Felt::from_hex_unchecked("0x9"),
        // h: u256
        Felt::from_hex_unchecked("0xa"),
        Felt::from_hex_unchecked("0x0"),
        // i: (i128, u128)
        Felt::from_hex_unchecked("0xb"),
        Felt::from_hex_unchecked("0xc"),
    ];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_external_struct_function_ambiguous_struct_name_cairo_expression_input() {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from(
        "
        BitArray { bit: 23 }, \
        BitArray { data: array![0], current: 1, read_pos: 2, write_pos: 3 }
        ",
    );

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("external_struct_fn").unwrap(),
    );

    result.unwrap_err().assert_contains(
        r#"Found more than one struct "BitArray" in ABI, please specify a full path to the item"#,
    );
}

#[tokio::test]
async fn test_happy_case_external_struct_function_cairo_expression_input() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from(indoc!(
            "
            data_transformer_contract::BitArray { bit: 23 }, \
            alexandria_data_structures::bit_array::BitArray { data: array![0], current: 1, read_pos: 2, write_pos: 3 }
            "
        ));

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("external_struct_fn").unwrap(),
    )?;

    let expected_output = [
        Felt::from_hex_unchecked("0x17"),
        Felt::from_hex_unchecked("0x1"),
        Felt::from_hex_unchecked("0x0"),
        Felt::from_hex_unchecked("0x1"),
        Felt::from_hex_unchecked("0x2"),
        Felt::from_hex_unchecked("0x3"),
    ];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_external_struct_function_invalid_path_to_external_struct() {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from(indoc!(
        "
        something::BitArray { bit: 23 }, \
        BitArray { data: array![0], current: 1, read_pos: 2, write_pos: 3 }
        "
    ));

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("external_struct_fn").unwrap(),
    );

    result
        .unwrap_err()
        .assert_contains(r#"Struct "something::BitArray" not found in ABI"#);
}

#[tokio::test]
async fn test_happy_case_contract_constructor() -> anyhow::Result<()> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    let input = String::from("0x123");

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("constructor").unwrap(),
    )?;

    let expected_output = [Felt::from_hex_unchecked("0x123")];

    assert_eq!(result, expected_output);

    Ok(())
}

#[tokio::test]
async fn test_external_enum_function_ambiguous_enum_name_cairo_expression_input() {
    // https://sepolia.starkscan.co/class/0x019ea00ebe2d83fb210fbd6f52c302b83c69e3c8c934f9404c87861e9d3aebbc#code
    let test_class_hash: Felt = Felt::from_hex_unchecked(
        "0x019ea00ebe2d83fb210fbd6f52c302b83c69e3c8c934f9404c87861e9d3aebbc",
    );

    let client = JsonRpcClient::new(HttpTransport::new(
        shared::test_utils::node_url::node_rpc_url(),
    ));

    let contract_class = client
        .get_class(BlockId::Tag(BlockTag::Latest), test_class_hash)
        .await
        .unwrap();

    let input = String::from(
        "
            TransactionState::Init() , \
            TransactionState::NotFound()
            ",
    );

    let result = Calldata::new(input).serialized(
        contract_class,
        &get_selector_from_name("external_enum_fn").unwrap(),
    );

    result.unwrap_err().assert_contains(
        r#"Found more than one enum "TransactionState" in ABI, please specify a full path to the item"#,
    );
}
