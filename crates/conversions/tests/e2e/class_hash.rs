#[cfg(test)]
mod tests_class_hash {
    use cairo_vm::utils::PRIME_STR;
    use conversions::string::{IntoDecStr, TryFromDecStr, TryFromHexStr};
    use conversions::{FromConv, IntoConv};
    use starknet_api::core::ClassHash;
    use starknet_api::core::{ContractAddress, EntryPointSelector, Nonce};
    use starknet_api::hash::StarkHash;
    use starknet_types_core::felt::Felt;

    #[test]
    fn test_class_hash_conversions_happy_case() {
        let felt = Felt::from_bytes_be(&[1u8; 32]);
        let class_hash = ClassHash(felt);

        assert_eq!(class_hash, ContractAddress::from_(class_hash).into_());
        assert_eq!(class_hash, Felt::from_(class_hash).into_());
        assert_eq!(class_hash, Felt::from_(class_hash).into_());
        assert_eq!(class_hash, Nonce::from_(class_hash).into_());
        assert_eq!(class_hash, EntryPointSelector::from_(class_hash).into_());
        assert_eq!(class_hash, StarkHash::from_(class_hash).into_());

        assert_eq!(
            class_hash,
            ClassHash::try_from_dec_str(&class_hash.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_class_hash_conversions_zero() {
        let felt = Felt::ZERO;
        let class_hash = ClassHash(felt);

        assert_eq!(class_hash, ContractAddress::from_(class_hash).into_());
        assert_eq!(class_hash, Felt::from_(class_hash).into_());
        assert_eq!(class_hash, Felt::from_(class_hash).into_());
        assert_eq!(class_hash, Nonce::from_(class_hash).into_());
        assert_eq!(class_hash, EntryPointSelector::from_(class_hash).into_());
        assert_eq!(class_hash, StarkHash::from_(class_hash).into_());

        assert_eq!(
            class_hash,
            ClassHash::try_from_dec_str(&class_hash.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_class_hash_conversions_limit() {
        let mut class_hash: ClassHash = Felt::MAX.into_();

        assert_eq!(class_hash, Felt::from_(class_hash).into_());
        assert_eq!(class_hash, Felt::from_(class_hash).into_());
        assert_eq!(class_hash, Nonce::from_(class_hash).into_());
        assert_eq!(class_hash, EntryPointSelector::from_(class_hash).into_());
        assert_eq!(class_hash, StarkHash::from_(class_hash).into_());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        let max_value = "0x07fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe";
        class_hash = ClassHash::try_from_hex_str(max_value).unwrap();
        assert_eq!(class_hash, ContractAddress::from_(class_hash).into_());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        let max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        class_hash = ClassHash::try_from_hex_str(max_value).unwrap();

        assert_eq!(
            class_hash,
            ClassHash::try_from_dec_str(&class_hash.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_class_hash_conversions_out_of_range() {
        assert!(ClassHash::try_from_hex_str(PRIME_STR).unwrap() == Felt::from(0_u8).into_());
    }
}
