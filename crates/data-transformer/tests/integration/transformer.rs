use crate::integration::{NO_CONSTRUCTOR_CLASS_HASH, get_abi, init_class};
use core::fmt;
use data_transformer::transform;
use indoc::indoc;
use itertools::Itertools;
use primitive_types::U256;
use starknet_rust::core::types::contract::AbiEntry;
use starknet_rust::core::types::{BlockId, BlockTag, ContractClass};
use starknet_rust::core::utils::get_selector_from_name;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use starknet_types_core::felt::Felt;
use std::ops::Not;
use test_case::test_case;

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
    let abi = get_abi().await;

    transform(
        input,
        &abi,
        &get_selector_from_name(selector).expect("should be valid selector"),
    )
}

#[tokio::test]
async fn test_function_not_found() {
    let selector = "nonexistent_fn";
    let result = run_transformer("('some_felt',)", selector).await;

    result.unwrap_err().assert_contains(
        format!(
            r#"Function with selector "{:#x}" not found in ABI of the contract"#,
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
    let abi = get_abi().await;

    let result = transform("Enum::Two(128)", &abi, &get_selector_from_name("enum_fn")?)?;

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
async fn test_happy_case_no_argument_function() {
    let result = run_transformer("", "no_args_fn").await.unwrap();

    let expected_output = [];

    assert_eq!(result, expected_output);
}

#[tokio::test]
async fn test_happy_case_implicit_contract_constructor() {
    let class = init_class(NO_CONSTRUCTOR_CLASS_HASH).await;
    let ContractClass::Sierra(sierra_class) = class else {
        panic!("Expected Sierra class, but got legacy Sierra class")
    };

    let abi: Vec<AbiEntry> = serde_json::from_str(sierra_class.abi.as_str()).unwrap();

    let result = transform("", &abi, &get_selector_from_name("constructor").unwrap()).unwrap();

    let expected_output = [];

    assert_eq!(result, expected_output);
}

fn option_complex_abi() -> Vec<AbiEntry> {
    serde_json::from_str(
        r#"[
            {
                "type": "enum",
                "name": "data_transformer_contract::EnumWithOption",
                "variants": [
                    {"name": "None", "type": "()"},
                    {"name": "Some", "type": "core::option::Option::<core::felt252>"}
                ]
            },
            {
                "type": "struct",
                "name": "data_transformer_contract::StructWithOption",
                "members": [
                    {"name": "a", "type": "core::felt252"},
                    {"name": "b", "type": "core::option::Option::<core::integer::u32>"}
                ]
            },
            {
                "type": "function",
                "name": "struct_with_option_fn",
                "inputs": [{"name": "a", "type": "data_transformer_contract::StructWithOption"}],
                "outputs": [],
                "state_mutability": "view"
            },
            {
                "type": "function",
                "name": "enum_with_option_fn",
                "inputs": [{"name": "a", "type": "data_transformer_contract::EnumWithOption"}],
                "outputs": [],
                "state_mutability": "view"
            },
            {
                "type": "function",
                "name": "option_of_enum_fn",
                "inputs": [{"name": "a", "type": "core::option::Option::<data_transformer_contract::EnumWithOption>"}],
                "outputs": [],
                "state_mutability": "view"
            }
        ]"#,
    )
    .unwrap()
}

#[test]
fn test_happy_case_struct_with_option_some() {
    let abi = option_complex_abi();
    let result = transform(
        "StructWithOption { a: 1, b: Option::Some(99) }",
        &abi,
        &get_selector_from_name("struct_with_option_fn").unwrap(),
    )
    .unwrap();

    // a=1, b=Some(99) -> [0, 99]
    assert_eq!(result, vec![Felt::ONE, Felt::ZERO, Felt::from(99u32)]);
}

#[test]
fn test_happy_case_struct_with_option_none() {
    let abi = option_complex_abi();
    let result = transform(
        "StructWithOption { a: 7, b: Option::None }",
        &abi,
        &get_selector_from_name("struct_with_option_fn").unwrap(),
    )
    .unwrap();

    // a=7, b=None -> [1]
    assert_eq!(result, vec![Felt::from(7u32), Felt::ONE]);
}

#[test]
fn test_happy_case_enum_with_option_variant_some() {
    let abi = option_complex_abi();
    let result = transform(
        "EnumWithOption::Some(Option::Some(42))",
        &abi,
        &get_selector_from_name("enum_with_option_fn").unwrap(),
    )
    .unwrap();

    // EnumWithOption::Some is variant 1, inner Option::Some(42) -> [0, 42]
    assert_eq!(result, vec![Felt::ONE, Felt::ZERO, Felt::from(42u32)]);
}

#[test]
fn test_happy_case_enum_with_option_variant_some_none() {
    let abi = option_complex_abi();
    let result = transform(
        "EnumWithOption::Some(Option::None)",
        &abi,
        &get_selector_from_name("enum_with_option_fn").unwrap(),
    )
    .unwrap();

    // EnumWithOption::Some is variant 1, inner Option::None -> [1]
    assert_eq!(result, vec![Felt::ONE, Felt::ONE]);
}

#[test]
fn test_happy_case_option_of_enum_some() {
    let abi = option_complex_abi();
    let result = transform(
        "Option::Some(EnumWithOption::Some(Option::Some(10)))",
        &abi,
        &get_selector_from_name("option_of_enum_fn").unwrap(),
    )
    .unwrap();

    // Option::Some -> [0], EnumWithOption::Some -> [1], Option::Some(10) -> [0, 10]
    assert_eq!(
        result,
        vec![Felt::ZERO, Felt::ONE, Felt::ZERO, Felt::from(10u32)]
    );
}

fn result_abi() -> Vec<AbiEntry> {
    serde_json::from_str(
        r#"[
            {
                "type": "function",
                "name": "result_fn",
                "inputs": [{"name": "a", "type": "core::result::Result::<core::integer::u32, core::felt252>"}],
                "outputs": [],
                "state_mutability": "view"
            }
        ]"#,
    )
    .unwrap()
}

#[test]
fn test_happy_case_result_err() {
    let abi = result_abi();
    let result = transform(
        "Result::Err(999)",
        &abi,
        &get_selector_from_name("result_fn").unwrap(),
    )
    .unwrap();

    // Err is variant 1, value is felt252
    assert_eq!(result, vec![Felt::ONE, Felt::from(999u32)]);
}

fn custom_generic_abi() -> Vec<AbiEntry> {
    serde_json::from_str(
        r#"[
            {
                "type": "struct",
                "name": "data_transformer_contract::Wrapper::<core::integer::u32>",
                "members": [
                    {"name": "value", "type": "core::integer::u32"}
                ]
            },
            {
                "type": "enum",
                "name": "data_transformer_contract::MaybeValue::<core::felt252>",
                "variants": [
                    {"name": "Nothing", "type": "()"},
                    {"name": "Just", "type": "core::felt252"}
                ]
            },
            {
                "type": "function",
                "name": "wrapper_fn",
                "inputs": [{"name": "a", "type": "data_transformer_contract::Wrapper::<core::integer::u32>"}],
                "outputs": [],
                "state_mutability": "view"
            },
            {
                "type": "function",
                "name": "maybe_value_fn",
                "inputs": [{"name": "a", "type": "data_transformer_contract::MaybeValue::<core::felt252>"}],
                "outputs": [],
                "state_mutability": "view"
            }
        ]"#,
    )
    .unwrap()
}

#[test]
fn test_happy_case_custom_generic_struct_full_path() {
    let abi = custom_generic_abi();
    let result = transform(
        "data_transformer_contract::Wrapper { value: 7 }",
        &abi,
        &get_selector_from_name("wrapper_fn").unwrap(),
    )
    .unwrap();

    assert_eq!(result, vec![Felt::from(7u32)]);
}

#[test]
fn test_happy_case_custom_generic_enum_unit_variant() {
    let abi = custom_generic_abi();
    let result = transform(
        "MaybeValue::Nothing",
        &abi,
        &get_selector_from_name("maybe_value_fn").unwrap(),
    )
    .unwrap();

    // Nothing is variant 0
    assert_eq!(result, vec![Felt::ZERO]);
}

#[test]
fn test_happy_case_custom_generic_enum_value_variant() {
    let abi = custom_generic_abi();
    let result = transform(
        "MaybeValue::Just(123)",
        &abi,
        &get_selector_from_name("maybe_value_fn").unwrap(),
    )
    .unwrap();

    // Just is variant 1
    assert_eq!(result, vec![Felt::ONE, Felt::from(123u32)]);
}

fn option_u32_abi() -> Vec<AbiEntry> {
    serde_json::from_str(
        r#"[
            {
                "type": "function",
                "name": "option_fn",
                "inputs": [{"name": "a", "type": "core::option::Option::<core::integer::u32>"}],
                "outputs": [{"type": "core::option::Option::<core::integer::u32>"}],
                "state_mutability": "view"
            }
        ]"#,
    )
    .unwrap()
}

#[test]
fn test_happy_case_option_none() {
    let abi = option_u32_abi();
    let result = transform(
        "Option::None",
        &abi,
        &get_selector_from_name("option_fn").unwrap(),
    )
    .unwrap();

    // In Cairo corelib Option: Some=0, None=1
    assert_eq!(result, vec![Felt::ONE]);
}

#[test]
fn test_happy_case_option_some() {
    let abi = option_u32_abi();
    let result = transform(
        "Option::Some(42)",
        &abi,
        &get_selector_from_name("option_fn").unwrap(),
    )
    .unwrap();

    // In Cairo corelib Option: Some=0, None=1; Some(42) serializes as [0, 42]
    assert_eq!(result, vec![Felt::ZERO, Felt::from(42u32)]);
}

// Option::None(x) — unit variant called with a value
#[test]
fn test_option_unit_variant_called_with_value() {
    let abi = option_u32_abi();
    let result = transform(
        "Option::None(99)",
        &abi,
        &get_selector_from_name("option_fn").unwrap(),
    );

    result.unwrap_err().assert_contains(
        r#"Variant "None" of "core::option::Option::<core::integer::u32>" takes no value"#,
    );
}

// Option::Some without parens — value-carrying variant used as unit
#[test]
fn test_option_value_variant_missing_value() {
    let abi = option_u32_abi();
    let result = transform(
        "Option::Some",
        &abi,
        &get_selector_from_name("option_fn").unwrap(),
    );

    result.unwrap_err().assert_contains(
        r#"Variant "Some" of "core::option::Option::<core::integer::u32>" takes no value"#,
    );
}

// MaybeValue::Nothing(x) — non-corelib unit variant called with a value
#[test]
fn test_custom_generic_enum_unit_variant_called_with_value() {
    let abi = custom_generic_abi();
    let result = transform(
        "MaybeValue::Nothing(1)",
        &abi,
        &get_selector_from_name("maybe_value_fn").unwrap(),
    );

    result
        .unwrap_err()
        .assert_contains(r#"Variant "Nothing" of "data_transformer_contract::MaybeValue::<core::felt252>" takes no value"#);
}

#[test]
fn test_option_wrong_variant() {
    let abi = option_u32_abi();
    let result = transform(
        "Option::Other",
        &abi,
        &get_selector_from_name("option_fn").unwrap(),
    );

    result
        .unwrap_err()
        .assert_contains(r#"Invalid variant "Other" for type"#);
}

// Option::Some(1, 2) — value-carrying variant called with wrong number of arguments
#[test]
fn test_option_some_too_many_args() {
    let abi = option_u32_abi();
    let result = transform(
        "Option::Some(1, 2)",
        &abi,
        &get_selector_from_name("option_fn").unwrap(),
    );

    result
        .unwrap_err()
        .assert_contains(r#"Variant "Some" of "core::option::Option::<core::integer::u32>" expects exactly 1 argument, got 2"#);
}

fn result_nested_abi() -> Vec<AbiEntry> {
    serde_json::from_str(
        r#"[
            {
                "type": "function",
                "name": "result_nested_fn",
                "inputs": [{"name": "a", "type": "core::result::Result::<core::option::Option::<core::integer::u32>, core::felt252>"}],
                "outputs": [],
                "state_mutability": "view"
            }
        ]"#,
    )
    .unwrap()
}

#[test]
fn test_happy_case_result_ok_with_option_some() {
    let abi = result_nested_abi();
    let result = transform(
        "Result::Ok(Option::Some(7))",
        &abi,
        &get_selector_from_name("result_nested_fn").unwrap(),
    )
    .unwrap();

    // Ok=0, Some=0, value=7
    assert_eq!(result, vec![Felt::ZERO, Felt::ZERO, Felt::from(7u32)]);
}

#[test]
fn test_happy_case_result_ok_with_option_none() {
    let abi = result_nested_abi();
    let result = transform(
        "Result::Ok(Option::None)",
        &abi,
        &get_selector_from_name("result_nested_fn").unwrap(),
    )
    .unwrap();

    // Ok=0, None=1
    assert_eq!(result, vec![Felt::ZERO, Felt::ONE]);
}

#[test]
fn test_happy_case_option_some_full_path() {
    let abi = option_u32_abi();
    let result = transform(
        "core::option::Option::Some(5)",
        &abi,
        &get_selector_from_name("option_fn").unwrap(),
    )
    .unwrap();

    // Some=0, value=5
    assert_eq!(result, vec![Felt::ZERO, Felt::from(5u32)]);
}

#[test]
fn test_happy_case_option_none_full_path() {
    let abi = option_u32_abi();
    let result = transform(
        "core::option::Option::None",
        &abi,
        &get_selector_from_name("option_fn").unwrap(),
    )
    .unwrap();

    assert_eq!(result, vec![Felt::ONE]);
}

#[test]
fn test_option_invalid_non_corelib_path() {
    let abi = option_u32_abi();
    let result = transform(
        "foo::Option::Some(5)",
        &abi,
        &get_selector_from_name("option_fn").unwrap(),
    );

    result
        .unwrap_err()
        .assert_contains(r#"Invalid argument type, expected "core::option::Option::<core::integer::u32>", got "foo::Option""#);
}

#[tokio::test]
async fn test_external_enum_function_ambiguous_enum_name_cairo_expression_input() {
    // https://sepolia.voyager.online/class/0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d#code
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

    let input = "
            TransactionState::Init() , \
            TransactionState::NotFound()
            ";

    let ContractClass::Sierra(sierra_class) = contract_class else {
        panic!("Expected Sierra class, but got legacy Sierra class")
    };

    let abi: Vec<AbiEntry> = serde_json::from_str(sierra_class.abi.as_str()).unwrap();

    let result = transform(
        input,
        &abi,
        &get_selector_from_name("external_enum_fn").unwrap(),
    );

    result.unwrap_err().assert_contains(
        r#"Found more than one enum "TransactionState" in ABI, please specify a full path to the item"#,
    );
}
