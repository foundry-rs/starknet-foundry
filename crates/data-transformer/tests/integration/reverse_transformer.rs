use crate::integration::get_abi;
use data_transformer::{reverse_transform_input, reverse_transform_output};
use itertools::Itertools;
use primitive_types::U256;
use starknet::core::utils::get_selector_from_name;
use starknet_types_core::felt::Felt;

async fn assert_reverse_transformation(
    input: &[Felt],
    selector: &str,
    expected_input: &str,
    expected_output: Option<&str>,
) {
    let abi = get_abi().await;
    let selector = get_selector_from_name(selector).unwrap();
    let result = reverse_transform_input(input, &abi, &selector).unwrap();
    assert_eq!(result, expected_input);

    let result = reverse_transform_output(input, &abi, &selector).unwrap();

    if let Some(expected_output) = expected_output {
        assert_eq!(result, expected_output);
    } else {
        // tests are written in a way that in most case the output is the same as the input
        // so passing None means we expect the output to be the same as the input
        assert_eq!(result, expected_input);
    }
}

#[tokio::test]
async fn test_unsigned() {
    assert_reverse_transformation(
        &[Felt::from(1_010_101_u32)],
        "unsigned_fn",
        "1010101_u32",
        None,
    )
    .await;
}

#[tokio::test]
async fn test_felt() {
    assert_reverse_transformation(
        &[Felt::from_hex_unchecked("0x64")],
        "simple_fn",
        "100_felt252",
        None,
    )
    .await;
}

#[tokio::test]
async fn test_u256_max() {
    assert_reverse_transformation(
        &[
            Felt::from_hex_unchecked("0xffffffffffffffffffffffffffffffff"),
            Felt::from_hex_unchecked("0xffffffffffffffffffffffffffffffff"),
        ],
        "u256_fn",
        &format!("{}_u256", U256::MAX),
        None,
    )
    .await;
}

#[tokio::test]
async fn test_u256() {
    assert_reverse_transformation(
        &[
            Felt::from_hex_unchecked("0x2137"),
            Felt::from_hex_unchecked("0x0"),
        ],
        "u256_fn",
        "8503_u256",
        None,
    )
    .await;
}

#[tokio::test]
async fn test_signed() {
    assert_reverse_transformation(&[Felt::from(-273i16)], "signed_fn", "-273_i32", None).await;
}

#[tokio::test]
async fn test_u32_max() {
    assert_reverse_transformation(
        &[Felt::from(u32::MAX)],
        "unsigned_fn",
        &format!("{}_u32", u32::MAX),
        None,
    )
    .await;
}

#[tokio::test]
async fn test_tuple_enum() {
    assert_reverse_transformation(
        &[
            Felt::from_hex_unchecked("0x859"),
            Felt::from_hex_unchecked("0x1"),
            Felt::from_hex_unchecked("0x0"),
        ],
        "tuple_fn",
        "(2137_felt252, 1_u8, Enum::One)",
        None,
    )
    .await;
}

#[tokio::test]
async fn test_tuple_enum_nested_struct() {
    assert_reverse_transformation(
        &[
            Felt::from(123),
            Felt::from(234),
            Felt::from(2),
            Felt::from(345),
            Felt::from(456),
        ],
        "tuple_fn",
        "(123_felt252, 234_u8, Enum::Three(NestedStructWithField { a: SimpleStruct { a: 345_felt252 }, b: 456_felt252 }))",
        None
    )
    .await;
}

#[tokio::test]
async fn test_happy_case_complex_function_cairo_expressions_input() {
    let max_u256 = U256::max_value().to_string();
    let expected = format!(
        "array![array![8503_felt252, 1056_felt252], array![1056_felt252, 8503_felt252]], 8_u8, -270_i16, \"some_string\", (35717030708670842322654162535_felt252, 100_u32), true, {max_u256}_u256"
    );

    let input = [
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

    assert_reverse_transformation(&input, "complex_fn", &expected, Some("")).await;
}

#[tokio::test]
async fn test_simple_struct() {
    assert_reverse_transformation(
        &[Felt::from_hex_unchecked("0x12")],
        "simple_struct_fn",
        "SimpleStruct { a: 18_felt252 }",
        None,
    )
    .await;
}

#[tokio::test]
async fn test_nested_struct() {
    assert_reverse_transformation(
        &[
            Felt::from_hex_unchecked("0x24"),
            Felt::from_hex_unchecked("0x60"),
        ],
        "nested_struct_fn",
        "NestedStructWithField { a: SimpleStruct { a: 36_felt252 }, b: 96_felt252 }",
        None,
    )
    .await;
}

#[tokio::test]
async fn test_span() {
    assert_reverse_transformation(
        &[
            Felt::from_hex_unchecked("0x3"),
            Felt::from_hex_unchecked("0x1"),
            Felt::from_hex_unchecked("0x2"),
            Felt::from_hex_unchecked("0x3"),
        ],
        "span_fn",
        "array![1_felt252, 2_felt252, 3_felt252].span()",
        None,
    )
    .await;
}

#[tokio::test]
async fn test_span_empty() {
    assert_reverse_transformation(&[Felt::ZERO], "span_fn", "array![].span()", None).await;
}

#[tokio::test]
async fn test_enum() {
    assert_reverse_transformation(&[Felt::ZERO], "enum_fn", "Enum::One", None).await;
}

#[tokio::test]
async fn test_enum_tuple() {
    assert_reverse_transformation(
        &[
            Felt::from_hex_unchecked("0x1"),
            Felt::from_hex_unchecked("0x80"),
        ],
        "enum_fn",
        "Enum::Two(128_u128)",
        None,
    )
    .await;
}

#[tokio::test]
async fn test_enum_nested_struct() {
    assert_reverse_transformation(
        &[
            Felt::from_hex_unchecked("0x2"),
            Felt::from_hex_unchecked("0x7b"),
            Felt::from_hex_unchecked("0xea"),
        ],
        "enum_fn",
        "Enum::Three(NestedStructWithField { a: SimpleStruct { a: 123_felt252 }, b: 234_felt252 })",
        None,
    )
    .await;
}

#[tokio::test]
async fn test_complex_struct() {
    let expected = r#"ComplexStruct { a: NestedStructWithField { a: SimpleStruct { a: 1_felt252 }, b: 2_felt252 }, b: 3_felt252, c: 4_u8, d: 5_i32, e: Enum::Two(6_u128), f: "seven", g: array![8_felt252, 9_felt252], h: 10_u256, i: (11_i128, 12_u128) }"#;

    let input = [
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

    assert_reverse_transformation(&input, "complex_struct_fn", expected, None).await;
}

#[tokio::test]
async fn test_external_type() {
    let input = [
        Felt::from_hex_unchecked("0x17"),
        Felt::from_hex_unchecked("0x1"),
        Felt::from_hex_unchecked("0x0"),
        Felt::from_hex_unchecked("0x1"),
        Felt::from_hex_unchecked("0x2"),
        Felt::from_hex_unchecked("0x3"),
    ];

    let expected = "BitArray { bit: 23_felt252 }, BitArray { data: array![CairoBytes31(0x0)], current: 1_felt252, read_pos: 2_u32, write_pos: 3_u32 }";

    assert_reverse_transformation(
        &input,
        "external_struct_fn",
        expected,
        Some(&format!("({expected})")),
    )
    .await;
}

#[tokio::test]
async fn test_constructor() {
    assert_reverse_transformation(
        &[Felt::from_hex_unchecked("0x123")],
        "constructor",
        "ContractAddress(0x123)",
        Some(""),
    )
    .await;
}

#[tokio::test]
async fn test_multiple_signed() {
    assert_reverse_transformation(
        &[Felt::from(124), Felt::from(97)],
        "multiple_signed_fn",
        "124_i32, 97_i8",
        Some(""),
    )
    .await;
}

#[tokio::test]
async fn test_multiple_signed_min() {
    assert_reverse_transformation(
        &[Felt::from(i32::MIN), Felt::from(i8::MIN)],
        "multiple_signed_fn",
        "-2147483648_i32, -128_i8",
        Some(""),
    )
    .await;
}

#[tokio::test]
async fn test_multiple_signed_max() {
    assert_reverse_transformation(
        &[Felt::from(i32::MAX), Felt::from(i8::MAX)],
        "multiple_signed_fn",
        "2147483647_i32, 127_i8",
        Some(""),
    )
    .await;
}
