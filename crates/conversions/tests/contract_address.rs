#[cfg(test)]
mod tests_contract_address {
    use conversions::StarknetConversions;
    use starknet_api::{
        core::{ContractAddress, PatriciaKey},
        hash::StarkFelt,
    };

    #[test]
    fn test_contract_address_conversions() {
        let felt: StarkFelt = StarkFelt::new([1u8; 32]).unwrap();
        let contract_address = ContractAddress(PatriciaKey::try_from(felt).unwrap());

        assert_eq!(
            contract_address,
            contract_address.to_class_hash().to_contract_address(),
        );
        assert_eq!(
            contract_address,
            contract_address.to_felt252().to_contract_address()
        );
        assert_eq!(
            contract_address,
            contract_address.to_field_element().to_contract_address()
        );
        assert_eq!(
            contract_address,
            contract_address.to_nonce().to_contract_address()
        );
        assert_eq!(
            contract_address,
            contract_address.to_short_string().to_contract_address()
        );
        assert_eq!(
            contract_address,
            contract_address.to_stark_felt().to_contract_address()
        );
        assert_eq!(
            contract_address,
            contract_address.to_stark_hash().to_contract_address()
        );
    }
}
