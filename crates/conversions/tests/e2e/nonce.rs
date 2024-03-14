#[cfg(test)]
mod tests_nonce {
    use cairo_felt::{Felt252, PRIME_STR};
    use conversions::string::{IntoDecStr, TryFromDecStr, TryFromHexStr};
    use conversions::{FromConv, IntoConv};
    use num_traits::Bounded;
    use starknet::core::types::FieldElement;
    use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
    use starknet_api::hash::{StarkFelt, StarkHash};

    #[test]
    fn test_nonce_conversions_happy_case() {
        let felt: StarkFelt = StarkFelt::new([1u8; 32]).unwrap();
        let nonce = Nonce(felt);

        assert_eq!(nonce, ClassHash::from_(nonce).into_());
        assert_eq!(nonce, ContractAddress::from_(nonce).into_());
        assert_eq!(nonce, Felt252::from_(nonce).into_());
        assert_eq!(nonce, FieldElement::from_(nonce).into_());
        assert_eq!(nonce, StarkFelt::from_(nonce).into_());
        assert_eq!(nonce, StarkHash::from_(nonce).into_());
        assert_eq!(nonce, EntryPointSelector::from_(nonce).into_());

        assert_eq!(
            nonce,
            Nonce::try_from_dec_str(&nonce.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_nonce_conversions_zero() {
        let felt: StarkFelt = StarkFelt::new([0u8; 32]).unwrap();
        let nonce = Nonce(felt);

        assert_eq!(nonce, ClassHash::from_(nonce).into_());
        assert_eq!(nonce, ContractAddress::from_(nonce).into_());
        assert_eq!(nonce, Felt252::from_(nonce).into_());
        assert_eq!(nonce, FieldElement::from_(nonce).into_());
        assert_eq!(nonce, StarkFelt::from_(nonce).into_());
        assert_eq!(nonce, StarkHash::from_(nonce).into_());
        assert_eq!(nonce, EntryPointSelector::from_(nonce).into_());

        assert_eq!(
            nonce,
            Nonce::try_from_dec_str(&nonce.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_nonce_conversions_limit() {
        let mut nonce: Nonce = Felt252::max_value().into_();

        assert_eq!(nonce, Felt252::from_(nonce).into_());
        assert_eq!(nonce, FieldElement::from_(nonce).into_());
        assert_eq!(nonce, ClassHash::from_(nonce).into_());
        assert_eq!(nonce, StarkFelt::from_(nonce).into_());
        assert_eq!(nonce, StarkHash::from_(nonce).into_());
        assert_eq!(nonce, EntryPointSelector::from_(nonce).into_());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        let max_value = "0x07fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe";
        nonce = Nonce::try_from_hex_str(max_value).unwrap();
        assert_eq!(nonce, ContractAddress::from_(nonce).into_());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        let max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        nonce = Nonce::try_from_hex_str(max_value).unwrap();

        assert_eq!(
            nonce,
            Nonce::try_from_dec_str(&nonce.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_nonce_conversions_out_of_range() {
        assert!(Nonce::try_from_hex_str(PRIME_STR).unwrap() == Felt252::from(0_u8).into_());
    }
}
