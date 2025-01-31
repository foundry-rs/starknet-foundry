use data_transformer::cairo_types::CairoU384;
use data_transformer::cairo_types::ParseCairoU384Error;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_from_str_valid_decimal() {
        let input = "123456789";
        let result = CairoU384::from_str(input).unwrap();
        let mut expected = [0u8; 48];
        expected[44..48].copy_from_slice(&[0x07, 0x5B, 0xCD, 0x15]);

        let expected = CairoU384::from_bytes(&expected);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_from_str_valid_hex() {
        let input = "0x1234567890abcdef";
        let result = CairoU384::from_str(input).unwrap();

        let mut expected = [0u8; 48];
        expected[40..48].copy_from_slice(&[0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF]);

        let expected = CairoU384::from_bytes(&expected);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_from_str_overflow() {
        let large_hex = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let result = CairoU384::from_str(large_hex);
        assert!(matches!(result, Err(ParseCairoU384Error::Overflow)));
    }

    #[test]
    fn test_from_str_invalid_input() {
        let invalid_hex = "0xghijkl";
        let result = CairoU384::from_str(invalid_hex);
        assert!(matches!(result, Err(ParseCairoU384Error::InvalidString(_))));
        let invalid_decimal = "123abc456";
        let result = CairoU384::from_str(invalid_decimal);
        assert!(matches!(result, Err(ParseCairoU384Error::InvalidString(_))));
        let empty = "";
        let result = CairoU384::from_str(empty);
        assert!(matches!(result, Err(ParseCairoU384Error::InvalidString(_))));
    }

    #[test]
    fn test_edge_cases() {
        let zero = "0";
        let result = CairoU384::from_str(zero).unwrap();
        let expected = CairoU384::from_bytes(&[0u8; 48]);
        assert_eq!(result, expected);
        let max_value = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        let result = CairoU384::from_str(max_value).unwrap();

        let mut expected = [0u8; 48];
        for i in 0..48 {
            expected[i] = 0xFF;
        }

        let expected = CairoU384::from_bytes(&expected);
        assert_eq!(result, expected);
    }
}
