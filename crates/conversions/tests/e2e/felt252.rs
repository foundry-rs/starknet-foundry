#[cfg(test)]
mod tests_felt252 {
    use crate::helpers::hex::str_hex_to_felt252;
    use cairo_felt::Felt252;
    use conversions::{FromConv, IntoConv};
    use starknet::core::types::FieldElement;
    use starknet_api::core::{ClassHash, ContractAddress, Nonce};
    use starknet_api::hash::{StarkFelt, StarkHash};

    #[test]
    fn test_felt252_conversions_happy_case() {
        let felt = Felt252::from(1u8);

        assert_eq!(felt, ClassHash::from_(felt.clone()).into_());
        assert_eq!(felt, ContractAddress::from_(felt.clone()).into_());
        assert_eq!(felt, FieldElement::from_(felt.clone()).into_());
        assert_eq!(felt, Nonce::from_(felt.clone()).into_());
        assert_eq!(felt, String::from_(felt.clone()).into_());
        assert_eq!(felt, StarkFelt::from_(felt.clone()).into_());
        assert_eq!(felt, StarkHash::from_(felt.clone()).into_());
    }

    #[test]
    fn test_felt252_conversions_zero() {
        let felt = Felt252::from(0u8);

        assert_eq!(felt, ClassHash::from_(felt.clone()).into_());
        assert_eq!(felt, ContractAddress::from_(felt.clone()).into_());
        assert_eq!(felt, FieldElement::from_(felt.clone()).into_());
        assert_eq!(felt, Nonce::from_(felt.clone()).into_());
        assert_eq!(felt, String::from_(felt.clone()).into_());
        assert_eq!(felt, StarkFelt::from_(felt.clone()).into_());
        assert_eq!(felt, StarkHash::from_(felt.clone()).into_());
    }

    #[test]
    fn test_felt252_conversions_limit() {
        // max_value from cairo_felt::PRIME_STR
        let mut max_value = "0x0800000000000011000000000000000000000000000000000000000000000000";
        let mut felt252 = str_hex_to_felt252(max_value);

        assert_eq!(felt252, Nonce::from_(felt252.clone()).into_());
        assert_eq!(felt252, FieldElement::from_(felt252.clone()).into_());
        assert_eq!(felt252, ClassHash::from_(felt252.clone()).into_());
        assert_eq!(felt252, StarkFelt::from_(felt252.clone()).into_());
        assert_eq!(felt252, StarkHash::from_(felt252.clone()).into_());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        max_value = "0x07ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        felt252 = str_hex_to_felt252(max_value);
        assert_eq!(felt252, ContractAddress::from_(felt252.clone()).into_());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        felt252 = str_hex_to_felt252(max_value);

        assert_eq!(felt252, String::from_(felt252.clone()).into_());
    }

    #[test]
    fn test_felt252_conversions_out_of_range() {
        // Can't set value bigger than max_value from cairo_felt::PRIME_STR
        // so we can't test all conversions.

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        let mut max_value = "0x0800000000000000000000000000000000000000000000000000000000000000";
        let mut felt252: Felt252 = str_hex_to_felt252(max_value);
        assert!(std::panic::catch_unwind(|| ContractAddress::from_(felt252)).is_err());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f80";
        felt252 = str_hex_to_felt252(max_value);
        assert!(std::panic::catch_unwind(|| String::from_(felt252)).is_err());
    }
}
