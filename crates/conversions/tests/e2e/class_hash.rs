#[cfg(test)]
mod tests_class_hash {
    use conversions::StarknetConversions;

    use crate::helpers::hex::str_hex_to_stark_felt;
    use starknet_api::{core::ClassHash, hash::StarkFelt};

    #[test]
    fn test_class_hash_conversions_happy_case() {
        let felt: StarkFelt = StarkFelt::new([1u8; 32]).unwrap();
        let class_hash = ClassHash(felt);

        assert_eq!(class_hash, class_hash.to_contract_address().to_class_hash());
        assert_eq!(class_hash, class_hash.to_felt252().to_class_hash());
        assert_eq!(class_hash, class_hash.to_field_element().to_class_hash());
        assert_eq!(class_hash, class_hash.to_nonce().to_class_hash());
        assert_eq!(class_hash, class_hash.to_short_string().to_class_hash());
        assert_eq!(class_hash, class_hash.to_stark_felt().to_class_hash());
        assert_eq!(class_hash, class_hash.to_stark_hash().to_class_hash());
    }

    #[test]
    fn test_class_hash_conversions_zero() {
        let felt: StarkFelt = StarkFelt::new([0u8; 32]).unwrap();
        let class_hash = ClassHash(felt);

        assert_eq!(class_hash, class_hash.to_contract_address().to_class_hash());
        assert_eq!(class_hash, class_hash.to_felt252().to_class_hash());
        assert_eq!(class_hash, class_hash.to_field_element().to_class_hash());
        assert_eq!(class_hash, class_hash.to_nonce().to_class_hash());
        assert_eq!(class_hash, class_hash.to_short_string().to_class_hash());
        assert_eq!(class_hash, class_hash.to_stark_felt().to_class_hash());
        assert_eq!(class_hash, class_hash.to_stark_hash().to_class_hash());
    }

    #[test]
    fn test_class_hash_conversions_limit() {
        // max_value from cairo_felt::PRIME_STR
        let mut max_value = "0x0800000000000011000000000000000000000000000000000000000000000000";
        let mut class_hash = ClassHash(str_hex_to_stark_felt(max_value));

        assert_eq!(class_hash, class_hash.to_felt252().to_class_hash());
        assert_eq!(class_hash, class_hash.to_field_element().to_class_hash());
        assert_eq!(class_hash, class_hash.to_nonce().to_class_hash());
        assert_eq!(class_hash, class_hash.to_stark_felt().to_class_hash());
        assert_eq!(class_hash, class_hash.to_stark_hash().to_class_hash());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        max_value = "0x07ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        class_hash = ClassHash(str_hex_to_stark_felt(max_value));
        assert_eq!(class_hash, class_hash.to_contract_address().to_class_hash());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        class_hash = ClassHash(str_hex_to_stark_felt(max_value));

        assert_eq!(class_hash, class_hash.to_short_string().to_class_hash());
    }

    #[test]
    fn test_class_hash_conversions_out_of_range() {
        // max_value from cairo_felt::PRIME_STR
        let mut max_value = "0x0800000000000011000000000000000000000000000000000000000000000001";
        let mut class_hash = ClassHash(str_hex_to_stark_felt(max_value));

        assert_ne!(class_hash, class_hash.to_felt252().to_class_hash());
        assert_ne!(class_hash, class_hash.to_field_element().to_class_hash());
        assert_ne!(class_hash, class_hash.to_nonce().to_class_hash());
        assert_ne!(class_hash, class_hash.to_stark_felt().to_class_hash());
        assert_ne!(class_hash, class_hash.to_stark_hash().to_class_hash());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        max_value = "0x0800000000000000000000000000000000000000000000000000000000000000";
        class_hash = ClassHash(str_hex_to_stark_felt(max_value));
        assert!(std::panic::catch_unwind(|| class_hash.to_contract_address()).is_err());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f80";
        class_hash = ClassHash(str_hex_to_stark_felt(max_value));
        assert!(std::panic::catch_unwind(|| class_hash.to_short_string()).is_err());
    }
}
