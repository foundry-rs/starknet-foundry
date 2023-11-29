#[cfg(test)]
mod tests_stark_felt {
    use crate::helpers::hex::str_hex_to_stark_felt;
    use cairo_felt::Felt252;
    use conversions::{FromConv, IntoConv};
    use starknet::core::types::FieldElement;
    use starknet_api::core::{ClassHash, ContractAddress, Nonce};
    use starknet_api::hash::StarkFelt;

    #[test]
    fn test_stark_felts_conversions_happy_case() {
        let stark_felt: StarkFelt = StarkFelt::new([1u8; 32]).unwrap();

        assert_eq!(stark_felt, ClassHash::from_(stark_felt).into_());
        assert_eq!(stark_felt, ContractAddress::from_(stark_felt).into_());
        assert_eq!(stark_felt, Felt252::from_(stark_felt).into_());
        assert_eq!(stark_felt, FieldElement::from_(stark_felt).into_());
        assert_eq!(stark_felt, String::from_(stark_felt).into_());
        assert_eq!(stark_felt, Nonce::from_(stark_felt).into_());
    }

    #[test]
    fn test_stark_felts_conversions_zero() {
        let stark_felt: StarkFelt = StarkFelt::new([0u8; 32]).unwrap();

        assert_eq!(stark_felt, ClassHash::from_(stark_felt).into_());
        assert_eq!(stark_felt, ContractAddress::from_(stark_felt).into_());
        assert_eq!(stark_felt, Felt252::from_(stark_felt).into_());
        assert_eq!(stark_felt, FieldElement::from_(stark_felt).into_());
        assert_eq!(stark_felt, String::from_(stark_felt).into_());
        assert_eq!(stark_felt, Nonce::from_(stark_felt).into_());
    }

    #[test]
    fn test_stark_felt_conversions_limit() {
        // max_value from cairo_felt::PRIME_STR
        let mut max_value = "0x0800000000000011000000000000000000000000000000000000000000000000";
        let mut stark_felt = str_hex_to_stark_felt(max_value);

        assert_eq!(stark_felt, ClassHash::from_(stark_felt).into_());
        assert_eq!(stark_felt, Felt252::from_(stark_felt).into_());
        assert_eq!(stark_felt, FieldElement::from_(stark_felt).into_());
        assert_eq!(stark_felt, Nonce::from_(stark_felt).into_());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        max_value = "0x07ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        stark_felt = str_hex_to_stark_felt(max_value);
        assert_eq!(stark_felt, ContractAddress::from_(stark_felt).into_());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        stark_felt = str_hex_to_stark_felt(max_value);

        assert_eq!(stark_felt, String::from_(stark_felt).into_());
    }

    #[test]
    fn test_stark_felt_conversions_out_of_range() {
        // max_value from cairo_felt::PRIME_STR
        let mut max_value = "0x0800000000000011000000000000000000000000000000000000000000000001";
        let mut stark_felt = str_hex_to_stark_felt(max_value);

        assert_eq!(stark_felt, ClassHash::from_(stark_felt).into_());
        assert_eq!(stark_felt, Felt252::from_(stark_felt).into_());
        assert_eq!(stark_felt, FieldElement::from_(stark_felt).into_());
        assert_eq!(stark_felt, Nonce::from_(stark_felt).into_());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        max_value = "0x0800000000000000000000000000000000000000000000000000000000000000";
        stark_felt = str_hex_to_stark_felt(max_value);
        assert!(std::panic::catch_unwind(|| ContractAddress::from_(stark_felt)).is_err());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f80";
        stark_felt = str_hex_to_stark_felt(max_value);
        assert!(std::panic::catch_unwind(|| String::from_(stark_felt)).is_err());
    }
}
