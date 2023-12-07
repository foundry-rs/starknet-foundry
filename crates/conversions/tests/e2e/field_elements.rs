#[cfg(test)]
mod tests_field_elements {
    use cairo_felt::{Felt252, PRIME_STR};
    use conversions::{FromConv, IntoConv, TryFromConv, TryIntoConv};
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
        assert_eq!(field_element, StarkFelt::from_(field_element).into_());
        assert_eq!(field_element, StarkHash::from_(field_element).into_());

        assert_eq!(
            field_element,
            String::from_(field_element).try_into_().unwrap()
        );
    }

    #[test]
    fn test_field_elements_conversions_zero() {
        let field_element = FieldElement::from(0u8);

        assert_eq!(field_element, ClassHash::from_(field_element).into_());
        assert_eq!(field_element, ContractAddress::from_(field_element).into_());
        assert_eq!(field_element, Felt252::from_(field_element).into_());
        assert_eq!(field_element, Nonce::from_(field_element).into_());
        assert_eq!(field_element, StarkFelt::from_(field_element).into_());
        assert_eq!(field_element, StarkHash::from_(field_element).into_());

        assert_eq!(
            field_element,
            String::from_(field_element).try_into_().unwrap()
        );
    }

    #[test]
    fn test_field_element_conversions_out_of_range() {
        let prime = String::from(PRIME_STR);
        assert!(FieldElement::try_from_(prime).is_err());
    }
}
