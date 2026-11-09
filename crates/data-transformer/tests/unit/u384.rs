use conversions::serde::deserialize::{BufferReadError, BufferReader, CairoDeserialize};
use conversions::serde::serialize::SerializeToFeltVec;
use data_transformer::cairo_types::CairoU384;
use data_transformer::cairo_types::ParseCairoU384Error;
use starknet_types_core::felt::Felt;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn test_from_bytes() {
        let mut input = [0u8; 48];
        input[36..48].copy_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4]); // limb_0
        input[24..36].copy_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]); // limb_1
        input[12..24].copy_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]); // limb_2
        input[0..12].copy_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]); // limb_3

        let first = CairoU384::from_bytes(&input);
        let second = CairoU384::from_bytes(&input);
        assert_eq!(first, second);
    }

    #[test]
    fn test_valid_decimal() {
        let input = "123456789";
        let result = CairoU384::from_str(input).unwrap();
        let mut expected = [0u8; 48];
        expected[44..48].copy_from_slice(&[0x07, 0x5B, 0xCD, 0x15]);

        let expected = CairoU384::from_bytes(&expected);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_valid_hex() {
        let input = "0x1234567890abcdef";
        let result = CairoU384::from_str(input).unwrap();

        let mut expected = [0u8; 48];
        expected[40..48].copy_from_slice(&[0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF]);

        let expected = CairoU384::from_bytes(&expected);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_overflow() {
        let large_hex = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let result = CairoU384::from_str(large_hex);
        assert!(matches!(result, Err(ParseCairoU384Error::Overflow)));
    }

    #[test]
    fn test_zero_value() {
        let zero = "0";
        let result = CairoU384::from_str(zero).unwrap();
        let expected = CairoU384::from_bytes(&[0u8; 48]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_max_value() {
        let max_value = "0xffffffffffffffffffffffffffffffff";
        let result = CairoU384::from_str(max_value).unwrap();

        let mut bytes = [0u8; 48];
        let start = 48 - max_value[2..].len() / 2;
        bytes[start..].fill(0xFF);
        let expected = CairoU384::from_bytes(&bytes);

        assert_eq!(result, expected);
    }

    /// `2^96 + 2`, the value from the `u384` calldata issues.
    const TWO_POW_96_PLUS_TWO: &str = "79228162514264337593543950338";

    #[test_case("0", Felt::ZERO, Felt::ZERO; "zero")]
    #[test_case("1", Felt::ONE, Felt::ZERO; "one")]
    fn serialize_matches_cairo_encoding(input: &str, low: Felt, high: Felt) {
        let value = CairoU384::from_str(input).unwrap();
        assert_eq!(value.serialize_to_vec(), vec![low, high]);
    }

    #[test]
    fn serialize_two_pow_96_plus_two_matches_cairo_encoding() {
        let value = CairoU384::from_str(TWO_POW_96_PLUS_TWO).unwrap();
        let packed = Felt::from_dec_str(TWO_POW_96_PLUS_TWO).unwrap();
        assert_eq!(value.serialize_to_vec(), vec![packed, Felt::ZERO]);
    }

    #[test]
    fn serialize_packs_each_limb_pair() {
        let mut bytes = [0u8; 48];
        bytes[47] = 4; // limb 0
        bytes[35] = 3; // limb 1
        bytes[23] = 2; // limb 2
        bytes[11] = 1; // limb 3
        let value = CairoU384::from_bytes(&bytes);

        let pow96 = Felt::from(1_u128 << 96);
        let felt0 = Felt::from(4_u128) + Felt::from(3_u128) * pow96;
        let felt1 = Felt::from(2_u128) + Felt::from(1_u128) * pow96;
        assert_eq!(value.serialize_to_vec(), vec![felt0, felt1]);
    }

    #[test]
    fn serialize_max_is_two_full_limb_pairs() {
        let value = CairoU384::from_bytes(&[0xff; 48]);
        let mut packed = [0u8; 32];
        packed[8..].fill(0xff);
        let felt = Felt::from_bytes_be(&packed);
        assert_eq!(value.serialize_to_vec(), vec![felt, felt]);
    }

    #[test_case("0"; "zero")]
    #[test_case("1"; "one")]
    #[test_case(TWO_POW_96_PLUS_TWO; "two_pow_96_plus_two")]
    #[test_case(
        "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        "max"
    )]
    fn round_trip(input: &str) {
        let value = CairoU384::from_str(input).unwrap();
        let felts = value.serialize_to_vec();
        assert_eq!(felts.len(), 2);

        let mut reader = BufferReader::new(&felts);
        let decoded = CairoU384::deserialize(&mut reader).unwrap();
        assert_eq!(decoded, value);
        assert_eq!(reader.remaining_len(), 0);
    }

    #[test]
    fn round_trip_value_with_high_limbs() {
        // 2 + 2^192. Cairo encodes it as felts [2, 1], not as four limbs.
        let mut bytes = [0u8; 48];
        bytes[47] = 2;
        bytes[23] = 1;
        let value = CairoU384::from_bytes(&bytes);

        let felts = value.serialize_to_vec();
        assert_eq!(felts, vec![Felt::from(2_u128), Felt::ONE]);

        let mut reader = BufferReader::new(&felts);
        assert_eq!(CairoU384::deserialize(&mut reader).unwrap(), value);
    }

    #[test]
    fn deserialize_known_cairo_encodings() {
        let one = [Felt::ONE, Felt::ZERO];
        let mut reader = BufferReader::new(&one);
        assert_eq!(
            CairoU384::deserialize(&mut reader).unwrap(),
            CairoU384::from_str("1").unwrap()
        );

        let packed = [Felt::from_dec_str(TWO_POW_96_PLUS_TWO).unwrap(), Felt::ZERO];
        let mut reader = BufferReader::new(&packed);
        assert_eq!(
            CairoU384::deserialize(&mut reader).unwrap(),
            CairoU384::from_str(TWO_POW_96_PLUS_TWO).unwrap()
        );
    }

    #[test]
    fn deserialize_two_felts_does_not_read_a_four_limb_encoding() {
        // [2, 1, 0, 0] is not `2^96 + 2`. Cairo reads the first two felts as `2^192 + 2`
        // and leaves the remaining zeros unread.
        let felts = [Felt::from(2_u128), Felt::ONE, Felt::ZERO, Felt::ZERO];
        let mut reader = BufferReader::new(&felts);
        let decoded = CairoU384::deserialize(&mut reader).unwrap();

        let mut bytes = [0u8; 48];
        bytes[47] = 2;
        bytes[23] = 1;
        assert_eq!(decoded, CairoU384::from_bytes(&bytes));
        assert_ne!(decoded, CairoU384::from_str(TWO_POW_96_PLUS_TWO).unwrap());
        assert_eq!(reader.remaining_len(), 2);
    }

    #[test]
    fn deserialize_leaves_felts_that_follow_the_value() {
        let mut felts = CairoU384::from_str("1").unwrap().serialize_to_vec();
        felts.push(Felt::from(7_u128));

        let mut reader = BufferReader::new(&felts);
        assert_eq!(
            CairoU384::deserialize(&mut reader).unwrap(),
            CairoU384::from_str("1").unwrap()
        );
        assert_eq!(reader.read_felt().unwrap(), Felt::from(7_u128));
        assert_eq!(reader.remaining_len(), 0);
    }

    #[test]
    fn deserialize_rejects_felt_wider_than_192_bits() {
        // 2^192 is the smallest integer that does not fit in two 96-bit limbs.
        let mut bytes = [0u8; 32];
        bytes[7] = 1;
        let too_wide = Felt::from_bytes_be(&bytes);
        let cases = [[too_wide, Felt::ZERO], [Felt::ZERO, too_wide]];

        for felts in cases {
            let error = CairoU384::deserialize(&mut BufferReader::new(&felts)).unwrap_err();
            assert!(matches!(error, BufferReadError::ParseFailed));
        }
    }

    #[test]
    fn deserialize_rejects_fewer_than_two_felts() {
        let error = CairoU384::deserialize(&mut BufferReader::new(&[Felt::ONE])).unwrap_err();
        assert!(matches!(error, BufferReadError::EndOfBuffer));

        let error = CairoU384::deserialize(&mut BufferReader::new(&[])).unwrap_err();
        assert!(matches!(error, BufferReadError::EndOfBuffer));
    }
}
