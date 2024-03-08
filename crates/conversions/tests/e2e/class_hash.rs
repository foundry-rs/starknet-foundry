#[cfg(test)]
mod tests_class_hash {
    use cairo_felt::{Felt252, PRIME_STR};
    use conversions::string::{IntoDecStr, TryFromDecStr, TryFromHexStr};
    use conversions::{FromConv, IntoConv};
    use starknet::core::types::FieldElement;
    use starknet_api::core::{ContractAddress, EntryPointSelector, Nonce};
    use starknet_api::hash::StarkHash;
    use starknet_api::{core::ClassHash, hash::StarkFelt};

    #[test]
    fn test_class_hash_conversions_happy_case() {
        let felt: StarkFelt = StarkFelt::new([1u8; 32]).unwrap();
        let class_hash = ClassHash(felt);

        assert_eq!(class_hash, ContractAddress::from_(class_hash).into_());
        assert_eq!(class_hash, Felt252::from_(class_hash).into_());
        assert_eq!(class_hash, FieldElement::from_(class_hash).into_());
        assert_eq!(class_hash, Nonce::from_(class_hash).into_());
        assert_eq!(class_hash, EntryPointSelector::from_(class_hash).into_());
        assert_eq!(class_hash, StarkFelt::from_(class_hash).into_());
        assert_eq!(class_hash, StarkHash::from_(class_hash).into_());

        assert_eq!(
            class_hash,
            ClassHash::try_from_dec_str(&class_hash.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_class_hash_conversions_zero() {
        let felt: StarkFelt = StarkFelt::new([0u8; 32]).unwrap();
        let class_hash = ClassHash(felt);

        assert_eq!(class_hash, ContractAddress::from_(class_hash).into_());
        assert_eq!(class_hash, Felt252::from_(class_hash).into_());
        assert_eq!(class_hash, FieldElement::from_(class_hash).into_());
        assert_eq!(class_hash, Nonce::from_(class_hash).into_());
        assert_eq!(class_hash, EntryPointSelector::from_(class_hash).into_());
        assert_eq!(class_hash, StarkFelt::from_(class_hash).into_());
        assert_eq!(class_hash, StarkHash::from_(class_hash).into_());

        assert_eq!(
            class_hash,
            ClassHash::try_from_dec_str(&class_hash.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_class_hash_conversions_limit() {
        // max_value from cairo_felt::PRIME_STR
        let mut max_value = "0x0800000000000011000000000000000000000000000000000000000000000000";
        let mut class_hash = ClassHash::try_from_hex_str(max_value).unwrap();

        assert_eq!(class_hash, Felt252::from_(class_hash).into_());
        assert_eq!(class_hash, FieldElement::from_(class_hash).into_());
        assert_eq!(class_hash, Nonce::from_(class_hash).into_());
        assert_eq!(class_hash, EntryPointSelector::from_(class_hash).into_());
        assert_eq!(class_hash, StarkFelt::from_(class_hash).into_());
        assert_eq!(class_hash, StarkHash::from_(class_hash).into_());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        max_value = "0x07ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        class_hash = ClassHash::try_from_hex_str(max_value).unwrap();
        assert_eq!(class_hash, ContractAddress::from_(class_hash).into_());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        class_hash = ClassHash::try_from_hex_str(max_value).unwrap();

        assert_eq!(
            class_hash,
            ClassHash::try_from_dec_str(&class_hash.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_class_hash_conversions_out_of_range() {
        assert!(ClassHash::try_from_hex_str(PRIME_STR).is_err());
    }
}
