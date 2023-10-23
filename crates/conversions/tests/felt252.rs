#[cfg(test)]
mod tests_felt252 {
    use cairo_felt::Felt252;
    use conversions::StarknetConversions;
    #[test]
    fn test_felt252_conversions() {
        let felt = Felt252::from(1u8);

        assert_eq!(felt, felt.to_class_hash().to_felt252());
        assert_eq!(felt, felt.to_contract_address().to_felt252());
        assert_eq!(felt, felt.to_field_element().to_felt252());
        assert_eq!(felt, felt.to_nonce().to_felt252());
        assert_eq!(felt, felt.to_short_string().to_felt252());
        assert_eq!(felt, felt.to_stark_felt().to_felt252());
        assert_eq!(felt, felt.to_stark_hash().to_felt252());
    }
}
