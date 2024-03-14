#[cfg(test)]
mod tests_stark_felt {
    use cairo_felt::{Felt252, PRIME_STR};
    use conversions::string::{IntoDecStr, TryFromDecStr, TryFromHexStr};
    use conversions::{FromConv, IntoConv};
    use num_traits::Bounded;
    use starknet::core::types::FieldElement;
    use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
    use starknet_api::hash::StarkFelt;

    #[test]
    fn test_stark_felts_conversions_happy_case() {
        let stark_felt: StarkFelt = StarkFelt::new([1u8; 32]).unwrap();

        assert_eq!(stark_felt, ClassHash::from_(stark_felt).into_());
        assert_eq!(stark_felt, ContractAddress::from_(stark_felt).into_());
        assert_eq!(stark_felt, Felt252::from_(stark_felt).into_());
        assert_eq!(stark_felt, FieldElement::from_(stark_felt).into_());
        assert_eq!(stark_felt, Nonce::from_(stark_felt).into_());
        assert_eq!(stark_felt, EntryPointSelector::from_(stark_felt).into_());

        assert_eq!(
            stark_felt,
            StarkFelt::try_from_dec_str(&stark_felt.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_stark_felts_conversions_zero() {
        let stark_felt: StarkFelt = StarkFelt::new([0u8; 32]).unwrap();

        assert_eq!(stark_felt, ClassHash::from_(stark_felt).into_());
        assert_eq!(stark_felt, ContractAddress::from_(stark_felt).into_());
        assert_eq!(stark_felt, Felt252::from_(stark_felt).into_());
        assert_eq!(stark_felt, FieldElement::from_(stark_felt).into_());
        assert_eq!(stark_felt, Nonce::from_(stark_felt).into_());
        assert_eq!(stark_felt, EntryPointSelector::from_(stark_felt).into_());

        assert_eq!(
            stark_felt,
            StarkFelt::try_from_dec_str(&stark_felt.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_stark_felt_conversions_limit() {
        let mut stark_felt: StarkFelt = Felt252::max_value().into_();

        assert_eq!(stark_felt, ClassHash::from_(stark_felt).into_());
        assert_eq!(stark_felt, Felt252::from_(stark_felt).into_());
        assert_eq!(stark_felt, FieldElement::from_(stark_felt).into_());
        assert_eq!(stark_felt, Nonce::from_(stark_felt).into_());
        assert_eq!(stark_felt, EntryPointSelector::from_(stark_felt).into_());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        let max_value = "0x07fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe";
        stark_felt = StarkFelt::try_from_hex_str(max_value).unwrap();
        assert_eq!(stark_felt, ContractAddress::from_(stark_felt).into_());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        let max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        stark_felt = StarkFelt::try_from_hex_str(max_value).unwrap();

        assert_eq!(
            stark_felt,
            StarkFelt::try_from_dec_str(&stark_felt.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_stark_felt_conversions_out_of_range() {
        assert!(StarkFelt::try_from_hex_str(PRIME_STR).unwrap() == Felt252::from(0_u8).into_());
    }
}
