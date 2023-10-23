#[cfg(test)]
mod tests_stark_felt {
    use conversions::StarknetConversions;
    use starknet_api::hash::StarkFelt;

    #[test]
    fn test_stark_felts_conversions() {
        let stark_felt: StarkFelt = StarkFelt::new([1u8; 32]).unwrap();

        assert_eq!(stark_felt, stark_felt.to_class_hash().to_stark_felt());
        assert_eq!(stark_felt, stark_felt.to_contract_address().to_stark_felt());
        assert_eq!(stark_felt, stark_felt.to_felt252().to_stark_felt());
        assert_eq!(stark_felt, stark_felt.to_field_element().to_stark_felt());
        assert_eq!(stark_felt, stark_felt.to_nonce().to_stark_felt());
        assert_eq!(stark_felt, stark_felt.to_short_string().to_stark_felt());
        assert_eq!(stark_felt, stark_felt.to_stark_hash().to_stark_felt());
    }
}
