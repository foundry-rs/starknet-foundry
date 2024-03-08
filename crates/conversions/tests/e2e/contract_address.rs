#[cfg(test)]
mod tests_contract_address {
    use cairo_felt::{Felt252, PRIME_STR};
    use conversions::string::{IntoDecStr, TryFromDecStr, TryFromHexStr};
    use conversions::{FromConv, IntoConv};
    use starknet::core::types::FieldElement;
    use starknet_api::core::{ClassHash, EntryPointSelector, Nonce};
    use starknet_api::hash::StarkHash;
    use starknet_api::{
        core::{ContractAddress, PatriciaKey},
        hash::StarkFelt,
    };

    #[test]
    fn test_contract_address_conversions_happy_case() {
        let felt: StarkFelt = StarkFelt::new([1u8; 32]).unwrap();
        let contract_address = ContractAddress(PatriciaKey::try_from(felt).unwrap());

        assert_eq!(contract_address, ClassHash::from_(contract_address).into_(),);
        assert_eq!(contract_address, Felt252::from_(contract_address).into_());
        assert_eq!(
            contract_address,
            FieldElement::from_(contract_address).into_()
        );
        assert_eq!(contract_address, Nonce::from_(contract_address).into_());
        assert_eq!(
            contract_address,
            EntryPointSelector::from_(contract_address).into_()
        );
        assert_eq!(contract_address, StarkFelt::from_(contract_address).into_());
        assert_eq!(contract_address, StarkHash::from_(contract_address).into_());

        assert_eq!(
            contract_address,
            ContractAddress::try_from_dec_str(&contract_address.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_contract_address_conversions_zero() {
        let felt: StarkFelt = StarkFelt::new([0u8; 32]).unwrap();
        let contract_address = ContractAddress(PatriciaKey::try_from(felt).unwrap());

        assert_eq!(contract_address, ClassHash::from_(contract_address).into_(),);
        assert_eq!(contract_address, Felt252::from_(contract_address).into_());
        assert_eq!(
            contract_address,
            FieldElement::from_(contract_address).into_()
        );
        assert_eq!(contract_address, Nonce::from_(contract_address).into_());
        assert_eq!(
            contract_address,
            EntryPointSelector::from_(contract_address).into_()
        );
        assert_eq!(contract_address, StarkFelt::from_(contract_address).into_());
        assert_eq!(contract_address, StarkHash::from_(contract_address).into_());

        assert_eq!(
            contract_address,
            ContractAddress::try_from_dec_str(&contract_address.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_contract_address_conversions_limit() {
        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        let mut max_value = "0x07ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let mut contract_address = ContractAddress::try_from_hex_str(max_value).unwrap();

        assert_eq!(contract_address, ClassHash::from_(contract_address).into_(),);
        assert_eq!(contract_address, Felt252::from_(contract_address).into_());
        assert_eq!(
            contract_address,
            FieldElement::from_(contract_address).into_()
        );
        assert_eq!(contract_address, Nonce::from_(contract_address).into_());
        assert_eq!(
            contract_address,
            EntryPointSelector::from_(contract_address).into_()
        );
        assert_eq!(contract_address, StarkFelt::from_(contract_address).into_());
        assert_eq!(contract_address, StarkHash::from_(contract_address).into_());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        contract_address = ContractAddress::try_from_hex_str(max_value).unwrap();

        assert_eq!(
            contract_address,
            ContractAddress::try_from_dec_str(&contract_address.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_contract_address_conversions_out_of_range() {
        assert!(ContractAddress::try_from_hex_str(PRIME_STR).is_err());
    }
}
