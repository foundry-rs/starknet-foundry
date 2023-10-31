#[cfg(test)]
mod tests_short_string {
    use conversions::StarknetConversions;

    #[test]
    fn test_short_strings_conversions_happy_case() {
        let short_string = "1".to_string();

        assert_eq!(short_string, short_string.to_class_hash().to_short_string());
        assert_eq!(
            short_string,
            short_string.to_contract_address().to_short_string()
        );
        assert_eq!(short_string, short_string.to_felt252().to_short_string());
        assert_eq!(
            short_string,
            short_string.to_field_element().to_short_string()
        );
        assert_eq!(short_string, short_string.to_nonce().to_short_string());
        assert_eq!(short_string, short_string.to_stark_felt().to_short_string());
        assert_eq!(short_string, short_string.to_stark_hash().to_short_string());
    }

    #[test]
    fn test_short_strings_conversions_zero() {
        let short_string = "0".to_string();

        assert_eq!(short_string, short_string.to_class_hash().to_short_string());
        assert_eq!(
            short_string,
            short_string.to_contract_address().to_short_string()
        );
        assert_eq!(short_string, short_string.to_felt252().to_short_string());
        assert_eq!(
            short_string,
            short_string.to_field_element().to_short_string()
        );
        assert_eq!(short_string, short_string.to_nonce().to_short_string());
        assert_eq!(short_string, short_string.to_stark_felt().to_short_string());
        assert_eq!(short_string, short_string.to_stark_hash().to_short_string());
    }

    #[test]
    fn test_short_string_conversions_limit() {
        // 31 characters.
        let short_string = "1234567890123456789012345678901".to_string();

        assert_eq!(short_string, short_string.to_class_hash().to_short_string());
        assert_eq!(short_string, short_string.to_felt252().to_short_string());
        assert_eq!(
            short_string,
            short_string.to_field_element().to_short_string()
        );
        assert_eq!(short_string, short_string.to_nonce().to_short_string());
        assert_eq!(short_string, short_string.to_stark_felt().to_short_string());
        assert_eq!(short_string, short_string.to_stark_hash().to_short_string());
        assert_eq!(
            short_string,
            short_string.to_contract_address().to_short_string()
        );
    }
}
