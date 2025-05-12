use crate::integration::get_abi;
use data_transformer::{reverse_transform_input, transform};
use primitive_types::U256;
use starknet::core::utils::get_selector_from_name;
use test_case::test_case;

#[test_case("1010101_u32", "unsigned_fn"; "u32")]
#[test_case(&format!("{}_u32", u32::MAX), "unsigned_fn"; "u32_max")]
#[test_case("0x64", "simple_fn"; "felt252")]
#[test_case(&format!("{}_u256", U256::MAX), "u256_fn"; "u256_max")]
#[test_case("8503_u256", "u256_fn"; "u256")]
#[test_case("-273_i32", "signed_fn"; "i32")]
#[test_case("-100_i32, -50_i8", "multiple_signed_fn"; "multiple_signed")]
#[test_case("(0x859, 1_u8, Enum::One)", "tuple_fn"; "tuple")]
#[test_case("(0x7b, 234_u8, Enum::Three(NestedStructWithField { a: SimpleStruct { a: 0x159 }, b: 0x1c8 }))", "tuple_fn"; "tuple_nested")]
#[test_case("array![array![0x2137, 0x420], array![0x420, 0x2137]], 8_u8, -270_i16, \"hello\", (0x2, 100_u32), true, 3_u256", "complex_fn"; "complex")]
#[test_case("SimpleStruct { a: 0x12 }", "simple_struct_fn"; "simple_struct")]
#[test_case("NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 0x60 }", "nested_struct_fn"; "nested_struct")]
#[test_case("array![0x1, 0x2, 0x3].span()", "span_fn"; "span")]
#[test_case("array![].span()", "span_fn"; "span_empty")]
#[test_case("Enum::One", "enum_fn"; "enum_no_data")]
#[test_case("Enum::Two(128_u128)", "enum_fn"; "enum_tuple")]
#[test_case("Enum::Three(NestedStructWithField { a: SimpleStruct { a: 0x7b }, b: 0xea })", "enum_fn"; "enum_nested")]
#[test_case(r#"ComplexStruct { a: NestedStructWithField { a: SimpleStruct { a: 0x1 }, b: 0x2 }, b: 0x3, c: 4_u8, d: 5_i32, e: Enum::Two(6_u128), f: "seven", g: array![0x8, 0x9], h: 10_u256, i: (11_i128, 12_u128) }"#, "complex_struct_fn"; "complex_struct")]
#[tokio::test]
async fn test_check_for_identity(calldata: &str, selector: &str) {
    let abi = get_abi().await;
    let selector = get_selector_from_name(selector).unwrap();

    let felts = transform(calldata, &abi, &selector).unwrap();

    let result = reverse_transform_input(&felts, &abi, &selector).unwrap();

    assert_eq!(result, calldata);
}
