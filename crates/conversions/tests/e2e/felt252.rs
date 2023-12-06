#[cfg(test)]
mod tests_felt252 {
    use crate::helpers::hex::str_hex_to_felt252;
    use cairo_felt::{Felt252, PRIME_STR};
    use cairo_lang_runner::short_string::as_cairo_short_string;
    use conversions::felt252::FromShortString;
    use conversions::{FromConv, IntoConv, TryFromConv, TryIntoConv};
    use starknet::core::types::FieldElement;
    use starknet_api::core::{ClassHash, ContractAddress, Nonce};
    use starknet_api::hash::{StarkFelt, StarkHash};

    #[test]
    fn test_felt252_conversions_happy_case() {
        let felt = Felt252::from(1u8);

        assert_eq!(felt, ClassHash::from_(felt.clone()).into_());
        assert_eq!(felt, ContractAddress::from_(felt.clone()).into_());
        assert_eq!(felt, FieldElement::from_(felt.clone()).into_());
        assert_eq!(felt, Nonce::from_(felt.clone()).into_());
        assert_eq!(felt, StarkFelt::from_(felt.clone()).into_());
        assert_eq!(felt, StarkHash::from_(felt.clone()).into_());

        assert_eq!(felt, String::from_(felt.clone()).try_into_().unwrap());
    }

    #[test]
    fn test_felt252_conversions_zero() {
        let felt = Felt252::from(0u8);

        assert_eq!(felt, ClassHash::from_(felt.clone()).into_());
        assert_eq!(felt, ContractAddress::from_(felt.clone()).into_());
        assert_eq!(felt, FieldElement::from_(felt.clone()).into_());
        assert_eq!(felt, Nonce::from_(felt.clone()).into_());
        assert_eq!(felt, StarkFelt::from_(felt.clone()).into_());
        assert_eq!(felt, StarkHash::from_(felt.clone()).into_());

        assert_eq!(felt, String::from_(felt.clone()).try_into_().unwrap());
    }

    #[test]
    fn test_felt252_conversions_limit() {
        // max_value from cairo_felt::PRIME_STR
        let mut max_value = "0x0800000000000011000000000000000000000000000000000000000000000000";
        let mut felt252 = str_hex_to_felt252(max_value);

        assert_eq!(felt252, Nonce::from_(felt252.clone()).into_());
        assert_eq!(felt252, FieldElement::from_(felt252.clone()).into_());
        assert_eq!(felt252, ClassHash::from_(felt252.clone()).into_());
        assert_eq!(felt252, StarkFelt::from_(felt252.clone()).into_());
        assert_eq!(felt252, StarkHash::from_(felt252.clone()).into_());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        max_value = "0x07ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        felt252 = str_hex_to_felt252(max_value);
        assert_eq!(felt252, ContractAddress::from_(felt252.clone()).into_());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        felt252 = str_hex_to_felt252(max_value);

        assert_eq!(felt252, String::from_(felt252.clone()).try_into_().unwrap());
    }

    #[test]
    fn test_felt252_try_from_string_out_of_range() {
        let prime = String::from(PRIME_STR);
        assert!(Felt252::try_from_(prime).is_err());
    }

    #[test]
    fn test_decimal_string() {
        let decimal_string = String::from("123456");
        let felt = Felt252::try_from_(decimal_string).unwrap();

        assert_eq!(felt, Felt252::from(123_456));
    }

    #[test]
    fn test_from_short_string() {
        let shortstring = String::from("abc");
        let felt = Felt252::from_short_string(&shortstring).unwrap();

        assert_eq!(shortstring, as_cairo_short_string(&felt).unwrap());
    }

    #[test]
    fn test_from_short_string_too_long() {
        // 32 characters long
        let shortstring = String::from("1234567890123456789012345678901a");

        assert!(Felt252::from_short_string(&shortstring).is_err());
    }
}
