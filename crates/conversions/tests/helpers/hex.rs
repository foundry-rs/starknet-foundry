use cairo_felt::Felt252;
use starknet::core::types::FieldElement;
use starknet_api::hash::StarkFelt;

#[must_use]
pub fn str_hex_to_stark_felt(hex_string: &str) -> StarkFelt {
    let bytes = hex_string_to_bytes(hex_string);
    StarkFelt::new(bytes).unwrap()
}

#[must_use]
pub fn str_hex_to_felt252(hex_string: &str) -> Felt252 {
    let bytes = hex_string_to_bytes(hex_string);
    Felt252::from_bytes_be(&bytes)
}

#[must_use]
pub fn str_hex_to_field_element(hex_string: &str) -> FieldElement {
    let bytes = hex_string_to_bytes(hex_string);
    FieldElement::from_bytes_be(&bytes).unwrap()
}

fn hex_string_to_bytes(hex_string: &str) -> [u8; 32] {
    let hex_string = hex_string.strip_prefix("0x").unwrap().to_string();
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        let byte_str = &hex_string[i * 2..i * 2 + 2];
        bytes[i] = u8::from_str_radix(byte_str, 16).unwrap();
    }
    bytes
}
