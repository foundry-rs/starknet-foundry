#[cfg(test)]
mod tests_nonce {
    use crate::helpers::hex::str_hex_to_stark_felt;
    use conversions::StarknetConversions;
    use starknet_api::core::Nonce;
    use starknet_api::hash::StarkFelt;

    #[test]
    fn test_nonce_conversions_happy_case() {
        let felt: StarkFelt = StarkFelt::new([1u8; 32]).unwrap();
        let nonce = Nonce(felt);

        assert_eq!(nonce, nonce.to_class_hash().to_nonce());
        assert_eq!(nonce, nonce.to_contract_address().to_nonce());
        assert_eq!(nonce, nonce.to_felt252().to_nonce());
        assert_eq!(nonce, nonce.to_field_element().to_nonce());
        assert_eq!(nonce, nonce.to_short_string().to_nonce());
        assert_eq!(nonce, nonce.to_stark_felt().to_nonce());
        assert_eq!(nonce, nonce.to_stark_hash().to_nonce());
    }

    #[test]
    fn test_nonce_conversions_zero() {
        let felt: StarkFelt = StarkFelt::new([0u8; 32]).unwrap();
        let nonce = Nonce(felt);

        assert_eq!(nonce, nonce.to_class_hash().to_nonce());
        assert_eq!(nonce, nonce.to_contract_address().to_nonce());
        assert_eq!(nonce, nonce.to_felt252().to_nonce());
        assert_eq!(nonce, nonce.to_field_element().to_nonce());
        assert_eq!(nonce, nonce.to_short_string().to_nonce());
        assert_eq!(nonce, nonce.to_stark_felt().to_nonce());
        assert_eq!(nonce, nonce.to_stark_hash().to_nonce());
    }

    #[test]
    fn test_nonce_conversions_limit() {
        // max_value from cairo_felt::PRIME_STR
        let mut max_value = "0x0800000000000011000000000000000000000000000000000000000000000000";
        let mut nonce = Nonce(str_hex_to_stark_felt(max_value));

        assert_eq!(nonce, nonce.to_felt252().to_nonce());
        assert_eq!(nonce, nonce.to_field_element().to_nonce());
        assert_eq!(nonce, nonce.to_class_hash().to_nonce());
        assert_eq!(nonce, nonce.to_stark_felt().to_nonce());
        assert_eq!(nonce, nonce.to_stark_hash().to_nonce());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        max_value = "0x07ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        nonce = Nonce(str_hex_to_stark_felt(max_value));
        assert_eq!(nonce, nonce.to_contract_address().to_nonce());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        nonce = Nonce(str_hex_to_stark_felt(max_value));

        assert_eq!(nonce, nonce.to_short_string().to_nonce());
    }

    #[test]
    fn test_nonce_conversions_out_of_range() {
        // Can't set value bigger than max_value from cairo_felt::PRIME_STR
        // so we can't test all conversions.

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        let mut max_value = "0x0800000000000000000000000000000000000000000000000000000000000000";
        let mut nonce = Nonce(str_hex_to_stark_felt(max_value));
        assert!(std::panic::catch_unwind(|| nonce.to_contract_address()).is_err());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f80";
        nonce = Nonce(str_hex_to_stark_felt(max_value));
        assert!(std::panic::catch_unwind(|| nonce.to_short_string()).is_err());
    }
}
