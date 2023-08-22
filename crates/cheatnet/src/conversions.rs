use blockifier::execution::execution_utils::felt_to_stark_felt;
use cairo_felt::Felt252;
use starknet::core::types::FieldElement;
use starknet_api::core::{ClassHash, ContractAddress, PatriciaKey};

#[must_use]
pub fn felt_from_short_string(short_str: &str) -> Felt252 {
    return Felt252::from_bytes_be(short_str.as_bytes());
}

#[must_use]
pub fn class_hash_from_felt(felt: &Felt252) -> ClassHash {
    return ClassHash(felt_to_stark_felt(felt));
}

#[must_use]
pub fn contract_address_from_felt(felt: &Felt252) -> ContractAddress {
    ContractAddress(
        PatriciaKey::try_from(felt_to_stark_felt(felt))
            .expect("StarkFelt to PatriciaKey conversion failed"),
    )
}

#[must_use]
pub fn field_element_to_felt252(field_element: &FieldElement) -> Felt252 {
    Felt252::from_bytes_be(&field_element.to_bytes_be())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parsing_felt_from_short_string() {
        let cases = [
            ("", Felt252::from(0)),
            ("{", Felt252::from(123)),
            ("PANIK", Felt252::from(344_693_033_291_u64)),
        ];

        for (str, felt_res) in cases {
            assert_eq!(felt_from_short_string(str), felt_res);
        }
    }
}
