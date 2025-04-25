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

// Deployment of contract from /tests/data/data_transformer
const TEST_CLASS_HASH: Felt =
    Felt::from_hex_unchecked("0x0786d1f010d66f838837290e472415186ba6a789fb446e7f92e444bed7b5d9c0");

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

async fn run_transformer(input: &str, selector: &str) -> anyhow::Result<Vec<Felt>> {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();

    Calldata::new(input.to_string()).serialized(
        contract_class,
        &get_selector_from_name(selector).expect("valid selector"),
    )
}

#[tokio::test]
async fn test_function_not_found() {
    let selector = "nonexistent_fn";
    let result = run_transformer("('some_felt',)", selector).await;

    result.unwrap_err().assert_contains(
        format!(
            r#"Function with selector "{}" not found in ABI of the contract"#,
            get_selector_from_name(selector).unwrap()
        )
        .as_str(),
    );
}

#[tokio::test]
async fn test_happy_case_numeric_type_suffix() {
    let result = run_transformer("1010101_u32", "unsigned_fn").await.unwrap();

    let expected_output = [Felt::from(1_010_101_u32)];

    assert_eq!(result, expected_output);
}

#[tokio::test]
async fn test_invalid_numeric_type_suffix() {
    let result = run_transformer("1_u10", "simple_fn").await;

    result
        .unwrap_err()
        .assert_contains(r#"Failed to parse value "1" into type "u10": unsupported type u10"#);
}

#[tokio::test]
async fn test_invalid_cairo_expression() {
    let result = run_transformer("(some_invalid_expression:,)", "simple_fn").await;

    result
        .unwrap_err()
        .assert_contains("Invalid Cairo expression found in input calldata");
}

#[tokio::test]
async fn test_invalid_argument_number() {
    let result = run_transformer("0x123, 'some_obsolete_argument', 10", "simple_fn").await;

    result
        .unwrap_err()
        .assert_contains("Invalid number of arguments: passed 3, expected 1");
}

#[tokio::test]
async fn test_happy_case_simple_cairo_expressions_input() {
    let result = run_transformer("100", "simple_fn").await.unwrap();

    let expected_output = [Felt::from_hex_unchecked("0x64")];

    assert_eq!(result, expected_output);
}

#[tokio::test]
async fn test_happy_case_u256_function_cairo_expressions_input_decimal() {
    let result = run_transformer(&format!("{}_u256", U256::MAX), "u256_fn")
        .await
        .unwrap();

    let expected_output = [
        Felt::from_hex_unchecked("0xffffffffffffffffffffffffffffffff"),
        Felt::from_hex_unchecked("0xffffffffffffffffffffffffffffffff"),
    ];

    assert_eq!(result, expected_output);
}

#[tokio::test]
async fn test_happy_case_u256_function_cairo_expressions_input_hex() {
    let result = run_transformer("0x2137_u256", "u256_fn").await.unwrap();

    let expected_output = [
        Felt::from_hex_unchecked("0x2137"),
        Felt::from_hex_unchecked("0x0"),
    ];

    assert_eq!(result, expected_output);
}

#[tokio::test]
async fn test_happy_case_signed_function_cairo_expressions_input() {
    let result = run_transformer("-273", "signed_fn").await.unwrap();

    let expected_output = [Felt::from(-273i16)];

    assert_eq!(result, expected_output);
}

// Problem: Although transformer fails to process the given input as `i32`, it then succeeds to interpret it as `felt252`
// Overflow checks will not work for functions having the same serialized and Cairo-like calldata length.
// User must provide a type suffix or get the invoke-time error
// Issue #2559
#[ignore = "Impossible to pass with the current solution"]
#[tokio::test]
async fn test_signed_fn_overflow() {
    let result = run_transformer(&format!("({},)", i32::MAX as u64 + 1), "signed_fn").await;

    result
        .unwrap_err()
        .assert_contains(r#"Failed to parse value "2147483648" into type "i32""#);
}

#[tokio::test]
async fn test_signed_fn_overflow_with_type_suffix() {
    let result = run_transformer(&format!("{}_i32", i32::MAX as u64 + 1), "signed_fn").await;

    result
        .unwrap_err()
        .assert_contains(r#"Failed to parse value "2147483648" into type "i32""#);
}

#[tokio::test]
async fn test_happy_case_unsigned_function_cairo_expressions_input() {
    let result = run_transformer(&format!("{}", u32::MAX), "unsigned_fn")
        .await
        .unwrap();

    let expected_output = [Felt::from(u32::MAX)];

    assert_eq!(result, expected_output);
}

#[tokio::test]
async fn test_happy_case_tuple_function_cairo_expression_input() {
    let result = run_transformer("(2137_felt252, 1_u8, Enum::One)", "tuple_fn")
        .await
        .unwrap();

    let expected_output = [
        Felt::from_hex_unchecked("0x859"),
        Felt::from_hex_unchecked("0x1"),
        Felt::from_hex_unchecked("0x0"),
    ];

    assert_eq!(result, expected_output);
}

#[tokio::test]
async fn test_happy_case_tuple_function_with_nested_struct_cairo_expression_input() {
    let result = run_transformer(
        "(123, 234, Enum::Three(NestedStructWithField {a: SimpleStruct {a: 345}, b: 456 }))",
        "tuple_fn",
    )
    .await
    .unwrap();

    let expected_output = [123, 234, 2, 345, 456]
        .into_iter()
        .map(Felt::from)
        .collect_vec();

    assert_eq!(result, expected_output);
}

#[tokio::test]
async fn test_happy_case_complex_function_cairo_expressions_input() {
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

    let result = run_transformer(&input, "complex_fn").await.unwrap();

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
}

#[tokio::test]
async fn test_happy_case_simple_struct_function_cairo_expression_input() {
    let result = run_transformer("SimpleStruct {a: 0x12}", "simple_struct_fn")
        .await
        .unwrap();

    let expected_output = [Felt::from_hex_unchecked("0x12")];

    assert_eq!(result, expected_output);
}

#[tokio::test]
async fn test_simple_struct_function_invalid_struct_argument() {
    let result = run_transformer(r#"SimpleStruct {a: "string"}"#, "simple_struct_fn").await;

    result
        .unwrap_err()
        .assert_contains(r#"Failed to parse value "string" into type "core::felt252""#);
}

#[tokio::test]
async fn test_simple_struct_function_invalid_struct_name() {
    let result = run_transformer("InvalidStructName {a: 0x10}", "simple_struct_fn").await;

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
    let result = run_transformer(input, "simple_struct_fn").await;

    result.unwrap_err().assert_contains(error_message);
}

#[tokio::test]
async fn test_happy_case_nested_struct_function_cairo_expression_input() {
    let result = run_transformer(
        "NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }",
        "nested_struct_fn",
    )
    .await
    .unwrap();

    let expected_output = [
        Felt::from_hex_unchecked("0x24"),
        Felt::from_hex_unchecked("0x60"),
    ];

    assert_eq!(result, expected_output);
}

#[tokio::test]
async fn test_happy_case_span_function_cairo_expression_input() {
    let result = run_transformer("array![1, 2, 3].span()", "span_fn")
        .await
        .unwrap();

    let expected_output = [
        Felt::from_hex_unchecked("0x3"),
        Felt::from_hex_unchecked("0x1"),
        Felt::from_hex_unchecked("0x2"),
        Felt::from_hex_unchecked("0x3"),
    ];

    assert_eq!(result, expected_output);
}

#[tokio::test]
async fn test_happy_case_empty_span_function_cairo_expression_input() {
    let result = run_transformer("array![].span()", "span_fn").await.unwrap();

    let expected_output = [Felt::from_hex_unchecked("0x0")];

    assert_eq!(result, expected_output);
}

#[tokio::test]
async fn test_span_function_array_input() {
    let result = run_transformer("array![1, 2, 3]", "span_fn").await;

    result
        .unwrap_err()
        .assert_contains(r#"Expected "core::array::Span::<core::felt252>", got array"#);
}

#[tokio::test]
async fn test_span_function_unsupported_method() {
    let result = run_transformer("array![1, 2, 3].into()", "span_fn").await;

    result
        .unwrap_err()
        .assert_contains(r#"Invalid function name, expected "span", got "into""#);
}

#[tokio::test]
async fn test_span_function_unsupported_operator() {
    let result = run_transformer("array![1, 2, 3]*span()", "span_fn").await;

    result
        .unwrap_err()
        .assert_contains(r#"Invalid operator, expected ".", got "*""#);
}

#[tokio::test]
async fn test_span_function_unsupported_right_hand_side() {
    let result = run_transformer("array![1, 2, 3].span", "span_fn").await;

    result
        .unwrap_err()
        .assert_contains(r#"Only calling ".span()" on "array![]" is supported, got "span""#);
}

#[tokio::test]
async fn test_span_function_unsupported_left_hand_side() {
    let result = run_transformer("(1, 2, 3).span", "span_fn").await;

    result.unwrap_err().assert_contains(
        r#"Only "array![]" is supported as left-hand side of "." operator, got "(1, 2, 3)""#,
    );
}

#[tokio::test]
async fn test_happy_case_enum_function_empty_variant_cairo_expression_input() {
    let result = run_transformer("Enum::One", "enum_fn").await.unwrap();

    let expected_output = [Felt::ZERO];

    assert_eq!(result, expected_output);
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
async fn test_happy_case_enum_function_nested_struct_variant_cairo_expression_input() {
    let result = run_transformer(
        "Enum::Three(NestedStructWithField { a: SimpleStruct { a: 123 }, b: 234 })",
        "enum_fn",
    )
    .await
    .unwrap();

    let expected_output = [
        Felt::from_hex_unchecked("0x2"),
        Felt::from_hex_unchecked("0x7b"),
        Felt::from_hex_unchecked("0xea"),
    ];

    assert_eq!(result, expected_output);
}

#[tokio::test]
async fn test_enum_function_invalid_variant_cairo_expression_input() {
    let result = run_transformer("Enum::InvalidVariant", "enum_fn").await;

    result
        .unwrap_err()
        .assert_contains(r#"Couldn't find variant "InvalidVariant" in enum "Enum""#);
}

#[tokio::test]
async fn test_happy_case_complex_struct_function_cairo_expression_input() {
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

    let result = run_transformer(data, "complex_struct_fn").await.unwrap();

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
}

#[tokio::test]
async fn test_external_struct_function_ambiguous_struct_name_cairo_expression_input() {
    let input = "
        BitArray { bit: 23 }, \
        BitArray { data: array![0], current: 1, read_pos: 2, write_pos: 3 }
        ";

    let result = run_transformer(input, "external_struct_fn").await;

    result.unwrap_err().assert_contains(
        r#"Found more than one struct "BitArray" in ABI, please specify a full path to the item"#,
    );
}

#[tokio::test]
async fn test_happy_case_external_struct_function_cairo_expression_input() {
    let input = indoc!(
            "
            data_transformer_contract::BitArray { bit: 23 }, \
            alexandria_data_structures::bit_array::BitArray { data: array![0], current: 1, read_pos: 2, write_pos: 3 }
            "
        );

    let result = run_transformer(input, "external_struct_fn").await.unwrap();

    let expected_output = [
        Felt::from_hex_unchecked("0x17"),
        Felt::from_hex_unchecked("0x1"),
        Felt::from_hex_unchecked("0x0"),
        Felt::from_hex_unchecked("0x1"),
        Felt::from_hex_unchecked("0x2"),
        Felt::from_hex_unchecked("0x3"),
    ];

    assert_eq!(result, expected_output);
}

#[tokio::test]
async fn test_external_struct_function_invalid_path_to_external_struct() {
    let input = indoc!(
        "
        something::BitArray { bit: 23 }, \
        BitArray { data: array![0], current: 1, read_pos: 2, write_pos: 3 }
        "
    );

    let result = run_transformer(input, "external_struct_fn").await;

    result
        .unwrap_err()
        .assert_contains(r#"Struct "something::BitArray" not found in ABI"#);
}

#[tokio::test]
async fn test_happy_case_contract_constructor() {
    let result = run_transformer("0x123", "constructor").await.unwrap();

    let expected_output = [Felt::from_hex_unchecked("0x123")];

    assert_eq!(result, expected_output);
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
