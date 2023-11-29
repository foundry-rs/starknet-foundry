#[cfg(test)]
mod tests_field_elements {
    use crate::helpers::hex::str_hex_to_field_element;
    use cairo_felt::Felt252;
    use conversions::{FromConv, IntoConv};
    use starknet::core::types::FieldElement;
    use starknet_api::core::{ClassHash, ContractAddress, Nonce};
    use starknet_api::hash::{StarkFelt, StarkHash};

    #[test]
    fn test_field_elements_conversions_happy_case() {
        let field_element = FieldElement::from(1u8);

        assert_eq!(field_element, ClassHash::from_(field_element).into_());
        assert_eq!(field_element, ContractAddress::from_(field_element).into_());
        assert_eq!(field_element, Felt252::from_(field_element).into_());
        assert_eq!(field_element, Nonce::from_(field_element).into_());
        assert_eq!(field_element, String::from_(field_element).into_());
        assert_eq!(field_element, StarkFelt::from_(field_element).into_());
        assert_eq!(field_element, StarkHash::from_(field_element).into_());
    }

    #[test]
    fn test_field_elements_conversions_zero() {
        let field_element = FieldElement::from(0u8);

        assert_eq!(field_element, ClassHash::from_(field_element).into_());
        assert_eq!(field_element, ContractAddress::from_(field_element).into_());
        assert_eq!(field_element, Felt252::from_(field_element).into_());
        assert_eq!(field_element, Nonce::from_(field_element).into_());
        assert_eq!(field_element, String::from_(field_element).into_());
        assert_eq!(field_element, StarkFelt::from_(field_element).into_());
        assert_eq!(field_element, StarkHash::from_(field_element).into_());
    }

    #[test]
    fn test_field_element_conversions_out_of_range() {
        // Can't set value bigger than max_value from cairo_felt::PRIME_STR
        // so we can't test all conversions.

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        let mut max_value = "0x0800000000000000000000000000000000000000000000000000000000000000";
        let mut field_element = str_hex_to_field_element(max_value);
        assert!(std::panic::catch_unwind(|| ContractAddress::from_(field_element)).is_err());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f80";
        field_element = str_hex_to_field_element(max_value);
        assert!(std::panic::catch_unwind(|| String::from_(field_element)).is_err());
    }
}
