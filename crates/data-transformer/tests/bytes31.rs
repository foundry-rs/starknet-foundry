use data_transformer::cairo_types::{CairoBytes31, ParseBytes31Error};
use starknet_types_core::felt::Felt;
use std::str::FromStr;

#[cfg(test)]
mod tests_bytes31 {
    use super::*;

    #[test]
    fn test_happy_case() {
        let bytes31 = CairoBytes31::from_str("0x123456789abcdef").unwrap();
        assert_eq!(
            Felt::from(bytes31),
            Felt::from_hex_unchecked("0x123456789abcdef")
        );
    }

    #[test]
    fn test_max_value() {
        let max_bytes31 = CairoBytes31::MAX;
        let from_str = CairoBytes31::from_str(
            "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        )
        .unwrap();
        assert_eq!(max_bytes31, from_str);
    }

    #[test]
    fn test_overflow() {
        let result = CairoBytes31::from_str(
            "0x1ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        );
        assert!(matches!(result, Err(ParseBytes31Error::Overflow)));
    }

    #[test]
    fn test_invalid_hex() {
        let result = CairoBytes31::from_str("invalid_hex");
        assert!(matches!(result, Err(ParseBytes31Error::CairoFromStrError)));
    }

    #[test]
    fn test_empty_string() {
        let result = CairoBytes31::from_str("");
        assert!(matches!(result, Err(ParseBytes31Error::CairoFromStrError)));
    }

    #[test]
    fn test_felt_conversion() {
        let bytes31 = CairoBytes31::from_str("0x123").unwrap();
        let felt: Felt = bytes31.into();
        assert_eq!(felt, Felt::from_hex_unchecked("0x123"));
    }

    #[test]
    fn test_zero_value() {
        let bytes31 = CairoBytes31::from_str("0x0").unwrap();
        assert_eq!(Felt::from(bytes31), Felt::from(0_u8));
    }

    #[test]
    fn test_leading_zeros() {
        let bytes31 = CairoBytes31::from_str("0x000123").unwrap();
        assert_eq!(Felt::from(bytes31), Felt::from_hex_unchecked("0x123"));
    }

    #[test]
    fn test_error_display() {
        assert_eq!(
            ParseBytes31Error::CairoFromStrError.to_string(),
            "Failed to parse as Cairo type"
        );
        assert_eq!(
            ParseBytes31Error::Overflow.to_string(),
            "Number is too large to fit in 31 bytes"
        );
    }
}
