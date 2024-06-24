#[cfg(test)]
mod tests_field_elements {
    use cairo_felt::{Felt252, PRIME_STR};
    use conversions::string::{IntoDecStr, TryFromDecStr, TryFromHexStr};
    use conversions::{FromConv, IntoConv};
    use starknet::core::types::FieldElement;
    use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
    use starknet_api::hash::{StarkFelt, StarkHash};

    #[test]
    fn test_field_elements_conversions_happy_case() {
        let field_element = FieldElement::from(1u8);

        assert_eq!(field_element, ClassHash::from_(field_element).into_());
        assert_eq!(field_element, ContractAddress::from_(field_element).into_());
        assert_eq!(field_element, Felt252::from_(field_element).into_());
        assert_eq!(field_element, Nonce::from_(field_element).into_());
        assert_eq!(
            field_element,
            EntryPointSelector::from_(field_element).into_()
        );
        assert_eq!(field_element, StarkFelt::from_(field_element).into_());
        assert_eq!(field_element, StarkHash::from_(field_element).into_());

        assert_eq!(
            field_element,
            FieldElement::try_from_dec_str(&field_element.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_field_elements_conversions_zero() {
        let field_element = FieldElement::from(0u8);

        assert_eq!(field_element, ClassHash::from_(field_element).into_());
        assert_eq!(field_element, ContractAddress::from_(field_element).into_());
        assert_eq!(field_element, Felt252::from_(field_element).into_());
        assert_eq!(field_element, Nonce::from_(field_element).into_());
        assert_eq!(
            field_element,
            EntryPointSelector::from_(field_element).into_()
        );
        assert_eq!(field_element, StarkFelt::from_(field_element).into_());
        assert_eq!(field_element, StarkHash::from_(field_element).into_());

        assert_eq!(
            field_element,
            FieldElement::try_from_dec_str(&field_element.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_field_element_conversions_out_of_range() {
        assert!(FieldElement::try_from_hex_str(PRIME_STR).unwrap() == Felt252::from(0_u8).into_());
    }
}
