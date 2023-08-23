use blockifier::execution::execution_utils::{felt_to_stark_felt, stark_felt_to_felt};
use cairo_felt::Felt252;
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::{ClassHash, ContractAddress, PatriciaKey};

#[must_use]
pub fn felt_selector_from_name(name: &str) -> Felt252 {
    let selector = get_selector_from_name(name).unwrap();
    Felt252::from_bytes_be(&selector.to_bytes_be())
}

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

#[must_use]
pub fn contract_address_from_felt252(felt: &Felt252) -> ContractAddress {
    ContractAddress(PatriciaKey::try_from(felt_to_stark_felt(felt)).unwrap())
}

#[cfg(test)]
mod test {
    use starknet_api::hash::StarkFelt;

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

    #[test]
    fn test_contract_address_to_felt() {
        let cases = [
            (ContractAddress::from(0_u8), Felt252::from(0)),
            (ContractAddress::from(123_u8), Felt252::from(123)),
        ];

        for (input, expected) in cases {
            assert_eq!(contract_address_to_felt(input), expected);
        }
    }

    #[test]
    fn test_class_hash_to_felt() {
        let cases = [
            (
                ClassHash(StarkFelt::new(Felt252::from(0).to_be_bytes()).unwrap()),
                Felt252::from(0),
            ),
            (
                ClassHash(StarkFelt::new(Felt252::from(123).to_be_bytes()).unwrap()),
                Felt252::from(123),
            ),
        ];

        for (input, expected) in cases {
            assert_eq!(class_hash_to_felt(input), expected);
        }
    }
}
