use starknet_api::hash::StarkFelt;
use cairo_felt::Felt252;
use starknet::core::types::FieldElement;

pub fn hex_to_stark_felt(hex_string: &str) -> StarkFelt {
    let hex_string = hex_string.strip_prefix("0x").unwrap().to_string();
    let mut byte_array = [0u8; 32];
    for i in 0..32 {
        let byte_str = &hex_string[i*2..i*2+2];
        byte_array[i] = u8::from_str_radix(byte_str, 16).unwrap();
    }

    StarkFelt::new(byte_array).unwrap()
}

pub fn hex_to_felt252(hex_string: &str) -> Felt252 {
    let hex_string = hex_string.strip_prefix("0x").unwrap().to_string();
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        let byte_str = &hex_string[i*2..i*2+2];
        bytes[i] = u8::from_str_radix(byte_str, 16).unwrap();
    }

    Felt252::from_bytes_be(&bytes)
}

pub fn hex_to_field_element(hex_string: &str) -> FieldElement {
    let hex_string = hex_string.strip_prefix("0x").unwrap().to_string();
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        let byte_str = &hex_string[i*2..i*2+2];
        bytes[i] = u8::from_str_radix(byte_str, 16).unwrap();
    }

    FieldElement::from_bytes_be(&bytes).unwrap()
}