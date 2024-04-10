#[cfg(test)]
mod tests_felt252 {
    use cairo_felt::{Felt252, PRIME_STR};
    use cairo_lang_runner::short_string::as_cairo_short_string;
    use conversions::byte_array::ByteArray;
    use conversions::felt252::{FromShortString, SerializeAsFelt252Vec};
    use conversions::string::{IntoDecStr, TryFromDecStr, TryFromHexStr};
    use conversions::{FromConv, IntoConv};
    use itertools::chain;
    use num_traits::Bounded;
    use starknet::core::types::FieldElement;
    use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
    use starknet_api::hash::{StarkFelt, StarkHash};

    #[test]
    fn test_felt252_conversions_happy_case() {
        let felt = Felt252::from(1u8);

        assert_eq!(felt, ClassHash::from_(felt.clone()).into_());
        assert_eq!(felt, ContractAddress::from_(felt.clone()).into_());
        assert_eq!(felt, FieldElement::from_(felt.clone()).into_());
        assert_eq!(felt, Nonce::from_(felt.clone()).into_());
        assert_eq!(felt, EntryPointSelector::from_(felt.clone()).into_());
        assert_eq!(felt, StarkFelt::from_(felt.clone()).into_());
        assert_eq!(felt, StarkHash::from_(felt.clone()).into_());

        assert_eq!(
            felt,
            Felt252::try_from_dec_str(&felt.clone().into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_felt252_conversions_zero() {
        let felt = Felt252::from(0u8);

        assert_eq!(felt, ClassHash::from_(felt.clone()).into_());
        assert_eq!(felt, ContractAddress::from_(felt.clone()).into_());
        assert_eq!(felt, FieldElement::from_(felt.clone()).into_());
        assert_eq!(felt, Nonce::from_(felt.clone()).into_());
        assert_eq!(felt, EntryPointSelector::from_(felt.clone()).into_());
        assert_eq!(felt, StarkFelt::from_(felt.clone()).into_());
        assert_eq!(felt, StarkHash::from_(felt.clone()).into_());

        assert_eq!(
            felt,
            Felt252::try_from_dec_str(&felt.clone().into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_felt252_conversions_limit() {
        let mut felt = Felt252::max_value();

        assert_eq!(felt, Nonce::from_(felt.clone()).into_());
        assert_eq!(felt, EntryPointSelector::from_(felt.clone()).into_());
        assert_eq!(felt, FieldElement::from_(felt.clone()).into_());
        assert_eq!(felt, ClassHash::from_(felt.clone()).into_());
        assert_eq!(felt, StarkFelt::from_(felt.clone()).into_());
        assert_eq!(felt, StarkHash::from_(felt.clone()).into_());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        let max_value = "0x07fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe";
        felt = Felt252::try_from_hex_str(max_value).unwrap();
        assert_eq!(felt, ContractAddress::from_(felt.clone()).into_());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        let max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        felt = Felt252::try_from_hex_str(max_value).unwrap();

        assert_eq!(
            felt,
            Felt252::try_from_dec_str(&felt.clone().into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_felt252_try_from_string_out_of_range() {
        assert!(Felt252::try_from_hex_str(PRIME_STR).unwrap() == Felt252::from(0_u8));
    }

    #[test]
    fn test_decimal_string() {
        let felt = Felt252::try_from_dec_str("123456").unwrap();

        assert_eq!(felt, Felt252::from(123_456));
    }

    #[test]
    fn test_from_short_string() {
        let felt = Felt252::from_short_string("abc").unwrap();

        assert_eq!("abc", &as_cairo_short_string(&felt).unwrap());
    }

    #[test]
    fn test_from_short_string_too_long() {
        // 32 characters long
        let shortstring = String::from("1234567890123456789012345678901a");

        assert!(Felt252::from_short_string(&shortstring).is_err());
    }

    #[test]
    fn test_result_to_felt252_vec() {
        let val: ByteArray = "a".into();
        let serialised_val = vec![Felt252::from(0), Felt252::from(97), Felt252::from(1)];

        let res: Result<ByteArray, ByteArray> = Ok(val.clone());
        let expected: Vec<Felt252> =
            chain!(vec![Felt252::from(0)], serialised_val.clone()).collect();
        assert_eq!(res.serialize_as_felt252_vec(), expected);

        let res: Result<ByteArray, ByteArray> = Err(val);
        let expected: Vec<Felt252> = chain!(vec![Felt252::from(1)], serialised_val).collect();
        assert_eq!(res.serialize_as_felt252_vec(), expected);
    }

    #[test]
    fn test_str_to_felt252_vec() {
        let val = "abc";
        assert_eq!(
            val.serialize_as_felt252_vec(),
            ByteArray::from(val).serialize_no_magic()
        );
    }
}
