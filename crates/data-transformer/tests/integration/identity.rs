use crate::integration::{CLASS, init_class};
use data_transformer::{reverse_transform_input, transform};
use primitive_types::U256;
use starknet::core::types::ContractClass;
use starknet::core::types::contract::AbiEntry;
use starknet::core::utils::get_selector_from_name;
use test_case::test_case;

#[test_case("1010101_u32", "unsigned_fn"; "u32")]
#[test_case(&format!("{}_u32", u32::MAX), "unsigned_fn"; "u32_max")]
#[test_case("100_felt252", "simple_fn"; "felt252")]
#[test_case(&format!("{}_u256", U256::MAX), "u256_fn"; "u256_max")]
#[test_case("8503_u256", "u256_fn"; "u256")]
#[test_case("-273_i32", "signed_fn"; "i32")]
#[test_case("-100_i32, -50_i8", "multiple_signed_fn"; "multiple_signed")]
#[test_case("(2137_felt252, 1_u8, Enum::One)", "tuple_fn"; "tuple")]
#[test_case("(123_felt252, 234_u8, Enum::Three(NestedStructWithField { a: SimpleStruct { a: 345_felt252 }, b: 456_felt252 }))", "tuple_fn"; "tuple_nested")]
#[test_case("array![array![8503_felt252, 1056_felt252], array![1056_felt252, 8503_felt252]], 8_u8, -270_i16, \"hello\", (2_felt252, 100_u32), true, 3_u256", "complex_fn"; "complex")]
#[test_case("SimpleStruct { a: 18_felt252 }", "simple_struct_fn"; "simple_struct")]
#[test_case("NestedStructWithField { a: SimpleStruct { a: 36_felt252 }, b: 96_felt252 }", "nested_struct_fn"; "nested_struct")]
#[test_case("array![1_felt252, 2_felt252, 3_felt252].span()", "span_fn"; "span")]
#[test_case("array![].span()", "span_fn"; "span_empty")]
#[test_case("Enum::One", "enum_fn"; "enum_no_data")]
#[test_case("Enum::Two(128_u128)", "enum_fn"; "enum_tuple")]
#[test_case("Enum::Three(NestedStructWithField { a: SimpleStruct { a: 123_felt252 }, b: 234_felt252 })", "enum_fn"; "enum_nested")]
#[test_case(r#"ComplexStruct { a: NestedStructWithField { a: SimpleStruct { a: 1_felt252 }, b: 2_felt252 }, b: 3_felt252, c: 4_u8, d: 5_i32, e: Enum::Two(6_u128), f: "seven", g: array![8_felt252, 9_felt252], h: 10_u256, i: (11_i128, 12_u128) }"#, "complex_struct_fn"; "complex_struct")]
#[tokio::test]
async fn test_check_for_identity(calldata: &str, selector: &str) {
    let contract_class = CLASS.get_or_init(init_class).await.to_owned();
    let ContractClass::Sierra(sierra_class) = contract_class.clone() else {
        panic!("Expected Sierra class, but got legacy Sierra class")
    };

    let abi: Vec<AbiEntry> = serde_json::from_str(sierra_class.abi.as_str()).unwrap();
    let selector = get_selector_from_name(selector).unwrap();

    let felts = transform(calldata, contract_class, &selector).unwrap();

    let result = reverse_transform_input(&felts, &abi, &selector).unwrap();

    assert_eq!(result, calldata);
}
