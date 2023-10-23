#[cfg(test)]
mod tests_stark_hash {
    use conversions::StarknetConversions;
    use starknet_api::hash::StarkHash;

    #[test]
    fn test_stark_hashs_conversions() {
        let stark_hash: StarkHash = StarkHash::new([1u8; 32]).unwrap();

        assert_eq!(stark_hash, stark_hash.to_class_hash().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_contract_address().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_felt252().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_field_element().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_nonce().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_short_string().to_stark_hash());
        assert_eq!(stark_hash, stark_hash.to_stark_hash().to_stark_hash());
    }
}
