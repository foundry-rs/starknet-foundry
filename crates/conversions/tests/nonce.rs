#[cfg(test)]
mod tests_nonce {
    use conversions::StarknetConversions;
    use starknet_api::core::Nonce;
    use starknet_api::hash::StarkFelt;

    #[test]
    fn test_nonces_conversions() {
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
}
