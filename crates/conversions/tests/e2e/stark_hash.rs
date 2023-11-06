#[cfg(test)]
mod tests_stark_hash {
    use crate::helpers::hex::str_hex_to_stark_felt;
    use conversions::StarknetConversions;
    use starknet_api::hash::StarkHash;

    #[test]
    fn test_stark_hashs_conversions_happy_case() {
        let stark_hash: StarkHash = StarkHash::new([1u8; 32]).unwrap();

        assert_eq!(stark_hash, stark_hash.to_class_hash().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_contract_address().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_felt252().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_field_element().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_nonce().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_short_string().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_stark_hash().to_stark_hash());
    }

    #[test]
    fn test_stark_hashs_conversions_zero() {
        let stark_hash: StarkHash = StarkHash::new([0u8; 32]).unwrap();

        assert_eq!(stark_hash, stark_hash.to_class_hash().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_contract_address().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_felt252().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_field_element().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_nonce().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_short_string().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_stark_hash().to_stark_hash());
    }

    #[test]
    fn test_stark_hash_conversions_limit() {
        // max_value from cairo_felt::PRIME_STR
        let mut max_value = "0x0800000000000011000000000000000000000000000000000000000000000000";
        let mut stark_hash = str_hex_to_stark_felt(max_value); //StarkHash is alias of StartFelt

        assert_eq!(stark_hash, stark_hash.to_felt252().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_field_element().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_nonce().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_class_hash().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_stark_felt().to_stark_hash());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        max_value = "0x07ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        stark_hash = str_hex_to_stark_felt(max_value);
        assert_eq!(stark_hash, stark_hash.to_contract_address().to_stark_hash());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        stark_hash = str_hex_to_stark_felt(max_value);

        assert_eq!(stark_hash, stark_hash.to_short_string().to_stark_hash());
    }

    #[test]
    fn test_stark_hash_conversions_out_of_range() {
        // max_value from cairo_felt::PRIME_STR
        let mut max_value = "0x0800000000000011000000000000000000000000000000000000000000000001";
        let mut stark_hash = str_hex_to_stark_felt(max_value);

        assert_ne!(stark_hash, stark_hash.to_felt252().to_stark_hash());
        assert_ne!(stark_hash, stark_hash.to_field_element().to_stark_hash());
        assert_ne!(stark_hash, stark_hash.to_nonce().to_stark_hash());
        assert_ne!(stark_hash, stark_hash.to_class_hash().to_stark_hash());
        assert_ne!(stark_hash, stark_hash.to_stark_felt().to_stark_hash());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        max_value = "0x0800000000000000000000000000000000000000000000000000000000000000";
        stark_hash = str_hex_to_stark_felt(max_value);
        assert!(std::panic::catch_unwind(|| stark_hash.to_contract_address()).is_err());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f80";
        stark_hash = str_hex_to_stark_felt(max_value);
        assert!(std::panic::catch_unwind(|| stark_hash.to_short_string()).is_err());
    }
}
