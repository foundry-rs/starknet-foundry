#[cfg(test)]
mod tests_felt {
    use cairo_vm::utils::PRIME_STR;
    use conversions::byte_array::ByteArray;
    use conversions::felt::FromShortString;
    use conversions::serde::serialize::SerializeToFeltVec;
    use conversions::string::{IntoDecStr, TryFromDecStr, TryFromHexStr};
    use conversions::{FromConv, IntoConv};
    use itertools::chain;
    use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
    use starknet_api::hash::StarkHash;
    use starknet_types_core::felt::Felt;

    #[test]
    fn test_felt_conversions_happy_case() {
        let felt = Felt::from(1u8);

        assert_eq!(felt, ClassHash::from_(felt).into_());
        assert_eq!(felt, ContractAddress::from_(felt).into_());
        assert_eq!(felt, Felt::from_(felt).into_());
        assert_eq!(felt, Nonce::from_(felt).into_());
        assert_eq!(felt, EntryPointSelector::from_(felt).into_());
        assert_eq!(felt, StarkHash::from_(felt).into_());

        assert_eq!(
            felt,
            Felt::try_from_dec_str(&felt.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_felt_conversions_zero() {
        let felt = Felt::from(0u8);

        assert_eq!(felt, ClassHash::from_(felt).into_());
        assert_eq!(felt, ContractAddress::from_(felt).into_());
        assert_eq!(felt, Felt::from_(felt).into_());
        assert_eq!(felt, Nonce::from_(felt).into_());
        assert_eq!(felt, EntryPointSelector::from_(felt).into_());
        assert_eq!(felt, StarkHash::from_(felt).into_());

        assert_eq!(
            felt,
            Felt::try_from_dec_str(&felt.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_felt_conversions_limit() {
        let mut felt = Felt::MAX;

        assert_eq!(felt, Nonce::from_(felt).into_());
        assert_eq!(felt, EntryPointSelector::from_(felt).into_());
        assert_eq!(felt, Felt::from_(felt).into_());
        assert_eq!(felt, ClassHash::from_(felt).into_());
        assert_eq!(felt, StarkHash::from_(felt).into_());

        // PATRICIA_KEY_UPPER_BOUND for contract_address from starknet_api-0.4.1/src/core.rs:156
        let max_value = "0x07fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe";
        felt = Felt::try_from_hex_str(max_value).unwrap();
        assert_eq!(felt, ContractAddress::from_(felt).into_());

        // Unknown source for this value, founded by try and error(cairo-lang-runner-2.2.0/src/short_string.rs).
        let max_value = "0x0777777777777777777777777777777777777f7f7f7f7f7f7f7f7f7f7f7f7f7f";
        felt = Felt::try_from_hex_str(max_value).unwrap();

        assert_eq!(
            felt,
            Felt::try_from_dec_str(&felt.into_dec_string()).unwrap()
        );
    }

    #[test]
    fn test_felt_try_from_string_out_of_range() {
        assert!(Felt::try_from_hex_str(PRIME_STR).unwrap() == Felt::from(0_u8));
    }

    #[test]
    fn test_decimal_string() {
        let felt = Felt::try_from_dec_str("123456").unwrap();

        assert_eq!(felt, Felt::from(123_456));
    }

    #[test]
    fn test_from_short_string() {
        let felt = Felt::from_short_string("abc").unwrap();

        assert_eq!(felt, Felt::from_hex("0x616263").unwrap());
    }

    #[test]
    fn test_from_short_string_too_long() {
        // 32 characters long
        let shortstring = String::from("1234567890123456789012345678901a");

        assert!(Felt::from_short_string(&shortstring).is_err());
    }

    #[test]
    fn test_result_to_felt_vec() {
        let val: ByteArray = "a".into();
        let serialised_val = vec![Felt::from(0), Felt::from(97), Felt::from(1)];

        let res: Result<ByteArray, ByteArray> = Ok(val.clone());
        let expected: Vec<Felt> = chain!(vec![Felt::from(0)], serialised_val.clone()).collect();
        assert_eq!(res.serialize_to_vec(), expected);

        let res: Result<ByteArray, ByteArray> = Err(val);
        let expected: Vec<Felt> = chain!(vec![Felt::from(1)], serialised_val).collect();
        assert_eq!(res.serialize_to_vec(), expected);
    }
}
