#[cfg(test)]
mod tests_entrypoint_selector {
    use cairo_vm::utils::PRIME_STR;
    use conversions::string::{IntoDecStr, TryFromDecStr, TryFromHexStr};
    use conversions::{FromConv, IntoConv};
    use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
    use starknet_api::hash::StarkHash;
    use starknet_types_core::felt::Felt;

    #[test]
    fn test_entrypoint_selector_conversions_happy_case() {
        let felt = Felt::from_bytes_be(&[1u8; 32]);
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
            Felt::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            Felt::from_(entrypoint_selector).into_()
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
            EntryPointSelector::try_from_dec_str(&entrypoint_selector.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_entrypoint_selector_conversions_zero() {
        let felt = Felt::ZERO;
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
            Felt::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            Felt::from_(entrypoint_selector).into_()
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
            EntryPointSelector::try_from_dec_str(&entrypoint_selector.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_entrypoint_selector_conversions_limit() {
        let mut entrypoint_selector: EntryPointSelector = Felt::MAX.into_();

        assert_eq!(
            entrypoint_selector,
            Felt::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            Felt::from_(entrypoint_selector).into_()
        );
        assert_eq!(
            entrypoint_selector,
            ClassHash::from_(entrypoint_selector).into_()
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
        let max_value = "0x07fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe";
        entrypoint_selector = EntryPointSelector::try_from_hex_str(max_value).unwrap();
        assert_eq!(
            entrypoint_selector,
            ContractAddress::from_(entrypoint_selector).into_()
        );

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        let max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        entrypoint_selector = EntryPointSelector::try_from_hex_str(max_value).unwrap();

        assert_eq!(
            entrypoint_selector,
            EntryPointSelector::try_from_dec_str(&entrypoint_selector.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_entrypoint_selector_conversions_out_of_range() {
        assert!(
            EntryPointSelector::try_from_hex_str(PRIME_STR).unwrap() == Felt::from(0_u8).into_()
        );
    }
}
