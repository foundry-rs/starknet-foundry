#[cfg(test)]
mod tests_cairo_u96 {
    use data_transformer::cairo_types::{CairoU96, ParseCairoU96Error};
    use starknet_types_core::felt::Felt;
    use std::str::FromStr;

    const U96_MAX: u128 = (2u128 << 96) - 1;

    #[test]
    fn test_valid_numbers() {
        let test_cases = [
            ("0", 0_u128),
            ("123", 123_u128),
            ("1000000", 1_000_000_u128),
            ("ff", 0xff_u128),
            ("1234abcd", 0x1234_abcd_u128),
        ];

        for (input, expected) in test_cases {
            let parsed = CairoU96::from_str(input).unwrap();
            assert_eq!(
                Felt::from(parsed),
                Felt::from(expected),
                "Failed parsing {input} - expected {expected}"
            );
        }
    }

    #[test]
    fn test_from_str_max_value() {
        let max_value_str = U96_MAX.to_string();
        let result = CairoU96::from_str(&max_value_str).unwrap();
        assert_eq!(Felt::from(result), Felt::from(U96_MAX));
        let max_value_hex = format!("{U96_MAX:x}");
        let result_hex = CairoU96::from_str(&max_value_hex).unwrap();
        assert_eq!(Felt::from(result_hex), Felt::from(U96_MAX));
    }

    #[test]
    fn test_from_str_invalid_input() {
        assert!(matches!(
            CairoU96::from_str("not_a_number"),
            Err(ParseCairoU96Error::InvalidString(_))
        ));

        assert!(matches!(
            CairoU96::from_str(""),
            Err(ParseCairoU96Error::InvalidString(_))
        ));
    }

    #[test]
    fn test_conversion_to_felt() {
        let values = [
            ("0", 0_u128),
            ("123", 123_u128),
            ("1000000", 1_000_000_u128),
            ("ff", 255_u128),
            (&U96_MAX.to_string(), U96_MAX),
        ];

        for (input, expected) in values {
            let cairo_u96 = CairoU96::from_str(input).unwrap();
            assert_eq!(Felt::from(cairo_u96), Felt::from(expected));
        }
    }
}
