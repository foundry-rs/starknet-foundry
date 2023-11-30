#[cfg(test)]
mod tests_nonce {
    use crate::helpers::hex::str_hex_to_stark_felt;
    use cairo_felt::{Felt252, PRIME_STR};
    use conversions::{FromConv, IntoConv, TryFromConv, TryIntoConv};
    use starknet::core::types::FieldElement;
    use starknet_api::core::{ClassHash, ContractAddress, Nonce};
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

        assert_eq!(nonce, String::from_(nonce).try_into_().unwrap());
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

        assert_eq!(nonce, String::from_(nonce).try_into_().unwrap());
    }

    #[test]
    fn test_nonce_conversions_limit() {
        // max_value from cairo_felt::PRIME_STR
        let mut max_value = "0x0800000000000011000000000000000000000000000000000000000000000000";
        let mut nonce = Nonce(str_hex_to_stark_felt(max_value));

        assert_eq!(nonce, Felt252::from_(nonce).into_());
        assert_eq!(nonce, FieldElement::from_(nonce).into_());
        assert_eq!(nonce, ClassHash::from_(nonce).into_());
        assert_eq!(nonce, StarkFelt::from_(nonce).into_());
        assert_eq!(nonce, StarkHash::from_(nonce).into_());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        max_value = "0x07ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        nonce = Nonce(str_hex_to_stark_felt(max_value));
        assert_eq!(nonce, ContractAddress::from_(nonce).into_());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        nonce = Nonce(str_hex_to_stark_felt(max_value));

        assert_eq!(nonce, String::from_(nonce).try_into_().unwrap());
    }

    #[test]
    fn test_nonce_conversions_out_of_range() {
        let prime = String::from(PRIME_STR);
        assert!(Nonce::try_from_(prime).is_err());
    }
}
