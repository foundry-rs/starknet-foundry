#[cfg(test)]
mod tests_felt252 {
    use crate::helpers::hex::str_hex_to_felt252;
    use cairo_felt::Felt252;
    use conversions::StarknetConversions;

    #[test]
    fn test_felt252_conversions_happy_case() {
        let felt = Felt252::from(1u8);

        assert_eq!(felt, felt.to_class_hash().to_felt252());
        assert_eq!(felt, felt.to_contract_address().to_felt252());
        assert_eq!(felt, felt.to_field_element().to_felt252());
        assert_eq!(felt, felt.to_nonce().to_felt252());
        assert_eq!(felt, felt.to_short_string().to_felt252());
        assert_eq!(felt, felt.to_stark_felt().to_felt252());
        assert_eq!(felt, felt.to_stark_hash().to_felt252());
    }

    #[test]
    fn test_felt252_conversions_zero() {
        let felt = Felt252::from(0u8);

        assert_eq!(felt, felt.to_class_hash().to_felt252());
        assert_eq!(felt, felt.to_contract_address().to_felt252());
        assert_eq!(felt, felt.to_field_element().to_felt252());
        assert_eq!(felt, felt.to_nonce().to_felt252());
        assert_eq!(felt, felt.to_short_string().to_felt252());
        assert_eq!(felt, felt.to_stark_felt().to_felt252());
        assert_eq!(felt, felt.to_stark_hash().to_felt252());
    }

    #[test]
    fn test_felt252_conversions_limit() {
        // max_value from cairo_felt::PRIME_STR
        let mut max_value = "0x0800000000000011000000000000000000000000000000000000000000000000";
        let mut felt252 = str_hex_to_felt252(max_value);

        assert_eq!(felt252, felt252.to_nonce().to_felt252());
        assert_eq!(felt252, felt252.to_field_element().to_felt252());
        assert_eq!(felt252, felt252.to_class_hash().to_felt252());
        assert_eq!(felt252, felt252.to_stark_felt().to_felt252());
        assert_eq!(felt252, felt252.to_stark_hash().to_felt252());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        max_value = "0x07ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        felt252 = str_hex_to_felt252(max_value);
        assert_eq!(felt252, felt252.to_contract_address().to_felt252());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        felt252 = str_hex_to_felt252(max_value);

        assert_eq!(felt252, felt252.to_short_string().to_felt252());
    }

    #[test]
    fn test_felt252_conversions_out_of_range() {
        // Can't set value bigger than max_value from cairo_felt::PRIME_STR
        // so we can't test all conversions.

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        let mut max_value = "0x0800000000000000000000000000000000000000000000000000000000000000";
        let mut felt252: Felt252 = str_hex_to_felt252(max_value);
        assert!(std::panic::catch_unwind(|| felt252.to_contract_address()).is_err());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f80";
        felt252 = str_hex_to_felt252(max_value);
        assert!(std::panic::catch_unwind(|| felt252.to_short_string()).is_err());
    }
}
