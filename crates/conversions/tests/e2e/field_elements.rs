#[cfg(test)]
mod tests_field_elements {
    use crate::helpers::hex::str_hex_to_field_element;
    use conversions::StarknetConversions;
    use starknet::core::types::FieldElement;

    #[test]
    fn test_field_elements_conversions_happy_case() {
        let field_element = FieldElement::from(1u8);

        assert_eq!(
            field_element,
            field_element.to_class_hash().to_field_element()
        );
        assert_eq!(
            field_element,
            field_element.to_contract_address().to_field_element()
        );
        assert_eq!(field_element, field_element.to_felt252().to_field_element());
        assert_eq!(field_element, field_element.to_nonce().to_field_element());
        assert_eq!(
            field_element,
            field_element.to_short_string().to_field_element()
        );
        assert_eq!(
            field_element,
            field_element.to_stark_felt().to_field_element()
        );
        assert_eq!(
            field_element,
            field_element.to_stark_hash().to_field_element()
        );
    }

    #[test]
    fn test_field_elements_conversions_zero() {
        let field_element = FieldElement::from(0u8);

        assert_eq!(
            field_element,
            field_element.to_class_hash().to_field_element()
        );
        assert_eq!(
            field_element,
            field_element.to_contract_address().to_field_element()
        );
        assert_eq!(field_element, field_element.to_felt252().to_field_element());
        assert_eq!(field_element, field_element.to_nonce().to_field_element());
        assert_eq!(
            field_element,
            field_element.to_short_string().to_field_element()
        );
        assert_eq!(
            field_element,
            field_element.to_stark_felt().to_field_element()
        );
        assert_eq!(
            field_element,
            field_element.to_stark_hash().to_field_element()
        );
    }

    #[test]
    fn test_field_element_conversions_out_of_range() {
        // Can't set value bigger than max_value from cairo_felt::PRIME_STR
        // so we can't test all conversions.

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        let mut max_value = "0x0800000000000000000000000000000000000000000000000000000000000000";
        let mut field_element = str_hex_to_field_element(max_value);
        assert!(std::panic::catch_unwind(|| field_element.to_contract_address()).is_err());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f80";
        field_element = str_hex_to_field_element(max_value);
        assert!(std::panic::catch_unwind(|| field_element.to_short_string()).is_err());
    }
}
