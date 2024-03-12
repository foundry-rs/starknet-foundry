#[cfg(test)]
mod tests_entrypoint_selector {
    use crate::helpers::hex::str_hex_to_stark_felt;
    use cairo_felt::{Felt252, PRIME_STR};
    use conversions::{FromConv, IntoConv, TryFromConv, TryIntoConv};
    use starknet::core::types::FieldElement;
    use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
    use starknet_api::hash::{StarkFelt, StarkHash};

    #[test]
    fn test_entrypoint_selector_conversions_happy_case() {
        let felt: StarkFelt = StarkFelt::new([1u8; 32]).unwrap();
        let entrypoint_selector = EntryPointSelector(felt);

        assert_eq!(
            entrypoint_selector,
            ClassHash::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            ContractAddress::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            Felt252::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            FieldElement::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            StarkFelt::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            StarkHash::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            Nonce::from_(entrypoint_selector).into_()
        );

        assert_eq!(
            entrypoint_selector,
            String::from_(entrypoint_selector).try_into_().unwrap()
        );
    }

    #[test]
    fn test_entrypoint_selector_conversions_zero() {
        let felt: StarkFelt = StarkFelt::new([0u8; 32]).unwrap();
        let entrypoint_selector = EntryPointSelector(felt);

        assert_eq!(
            entrypoint_selector,
            ClassHash::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            ContractAddress::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            Felt252::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            FieldElement::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            StarkFelt::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            StarkHash::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            Nonce::from_(entrypoint_selector).into_()
        );

        assert_eq!(
            entrypoint_selector,
            String::from_(entrypoint_selector).try_into_().unwrap()
        );
    }

    #[test]
    fn test_entrypoint_selector_conversions_limit() {
        // max_value from cairo_felt::PRIME_STR
        let mut max_value = "0x0800000000000011000000000000000000000000000000000000000000000000";
        let mut entrypoint_selector = EntryPointSelector(str_hex_to_stark_felt(max_value));

        assert_eq!(
            entrypoint_selector,
            Felt252::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            FieldElement::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            ClassHash::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            StarkFelt::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            StarkHash::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            Nonce::from_(entrypoint_selector).into_()
        );

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        max_value = "0x07ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        entrypoint_selector = EntryPointSelector(str_hex_to_stark_felt(max_value));
        assert_eq!(
            entrypoint_selector,
            ContractAddress::from_(entrypoint_selector).into_()
        );

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        entrypoint_selector = EntryPointSelector(str_hex_to_stark_felt(max_value));

        assert_eq!(
            entrypoint_selector,
            String::from_(entrypoint_selector).try_into_().unwrap()
        );
    }

    #[test]
    fn test_entrypoint_selector_conversions_out_of_range() {
        let prime = String::from(PRIME_STR);
        assert!(EntryPointSelector::try_from_(prime).is_err());
    }
}
