#[cfg(test)]
mod tests_class_hash {
    use conversions::StarknetConversions;

    use starknet_api::{core::ClassHash, hash::StarkFelt};

    #[test]
    fn test_class_hash_conversions() {
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
}
