#[cfg(test)]
mod tests_field_elements {
    use conversions::StarknetConversions;
    use starknet::core::types::FieldElement;

    #[test]
    fn test_field_elements_conversions() {
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
}
