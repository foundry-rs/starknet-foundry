#[cfg(test)]
mod tests_short_string {
    use cairo_felt::Felt252;
    use conversions::{FromConv, TryFromConv};
    use starknet::core::types::FieldElement;
    use starknet_api::core::{ClassHash, ContractAddress, Nonce};
    use starknet_api::hash::{StarkFelt, StarkHash};

    #[test]
    fn test_short_strings_conversions_happy_case() {
        let short_string = "1".to_string();

        assert_eq!(
            short_string,
            String::from_(ClassHash::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(ContractAddress::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(Felt252::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(FieldElement::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(Nonce::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(StarkFelt::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(StarkHash::try_from_(short_string.clone()).unwrap())
        );
    }

    #[test]
    fn test_short_strings_conversions_zero() {
        let short_string = "0".to_string();

        assert_eq!(
            short_string,
            String::from_(ClassHash::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(ContractAddress::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(Felt252::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(FieldElement::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(Nonce::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(StarkFelt::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(StarkHash::try_from_(short_string.clone()).unwrap())
        );
    }

    #[test]
    fn test_short_string_conversions_limit() {
        // 31 characters.
        let short_string = "1234567890123456789012345678901".to_string();

        assert_eq!(
            short_string,
            String::from_(ClassHash::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(Felt252::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(FieldElement::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(Nonce::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(StarkFelt::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(StarkHash::try_from_(short_string.clone()).unwrap())
        );
        assert_eq!(
            short_string,
            String::from_(ContractAddress::try_from_(short_string.clone()).unwrap())
        );
    }
}
