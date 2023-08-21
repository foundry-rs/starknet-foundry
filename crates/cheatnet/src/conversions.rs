use blockifier::execution::execution_utils::stark_felt_to_felt;
use cairo_felt::Felt252;
use starknet_api::core::{ClassHash, ContractAddress};

#[must_use]
pub fn felt_from_short_string(short_str: &str) -> Felt252 {
    return Felt252::from_bytes_be(short_str.as_bytes());
}

#[must_use]
pub fn contract_address_to_felt(contract_address: ContractAddress) -> Felt252 {
    stark_felt_to_felt(*contract_address.0.key())
}

#[must_use]
pub fn class_hash_to_felt(class_hash: ClassHash) -> Felt252 {
    stark_felt_to_felt(class_hash.0)
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
