#[cfg(test)]
mod tests_contract_address {
    use crate::helpers::hex::str_hex_to_stark_felt;
    use cairo_felt::{Felt252, PRIME_STR};
    use conversions::{FromConv, IntoConv, TryFromConv, TryIntoConv};
    use starknet::core::types::FieldElement;
    use starknet_api::core::{ClassHash, Nonce};
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
        assert_eq!(contract_address, StarkFelt::from_(contract_address).into_());
        assert_eq!(contract_address, StarkHash::from_(contract_address).into_());

        assert_eq!(
            contract_address,
            String::from_(contract_address).try_into_().unwrap()
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
        assert_eq!(contract_address, StarkFelt::from_(contract_address).into_());
        assert_eq!(contract_address, StarkHash::from_(contract_address).into_());

        assert_eq!(
            contract_address,
            String::from_(contract_address).try_into_().unwrap()
        );
    }

    #[test]
    fn test_contract_address_conversions_limit() {
        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        let mut max_value = "0x07ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let mut contract_address =
            ContractAddress(PatriciaKey::try_from(str_hex_to_stark_felt(max_value)).unwrap());

        assert_eq!(contract_address, ClassHash::from_(contract_address).into_(),);
        assert_eq!(contract_address, Felt252::from_(contract_address).into_());
        assert_eq!(
            contract_address,
            FieldElement::from_(contract_address).into_()
        );
        assert_eq!(contract_address, Nonce::from_(contract_address).into_());
        assert_eq!(contract_address, StarkFelt::from_(contract_address).into_());
        assert_eq!(contract_address, StarkHash::from_(contract_address).into_());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        contract_address =
            ContractAddress(PatriciaKey::try_from(str_hex_to_stark_felt(max_value)).unwrap());

        assert_eq!(
            contract_address,
            String::from_(contract_address).try_into_().unwrap()
        );
    }

    #[test]
    fn test_contract_address_conversions_out_of_range() {
        let prime = String::from(PRIME_STR);
        assert!(ContractAddress::try_from_(prime).is_err());
    }
}
