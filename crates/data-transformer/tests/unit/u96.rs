#[cfg(test)]
mod tests_cairo_u96 {
    use data_transformer::cairo_types::{CairoU96, ParseCairoU96Error};
    use starknet_types_core::felt::Felt;
    use std::str::FromStr;

    use test_case::test_case;

    const U96_MAX: u128 = (2u128 << 96) - 1;

    #[test_case("0", 0_u128 ; "zero")]
    #[test_case("123", 123_u128 ; "small decimal")]
    #[test_case("1000000", 1_000_000_u128 ; "million")]
    #[test_case("ff", 0xff_u128 ; "small hex")]
    #[test_case("1234abcd", 0x1234_abcd_u128 ; "large hex")]
    fn test_valid_numbers(input: &str, expected: u128) {
        let parsed = CairoU96::from_str(input).unwrap();
        assert_eq!(
            Felt::from(parsed),
            Felt::from(expected),
            "Failed parsing {input} - expected {expected}"
        );
    }

    #[test]
    fn test_max_value() {
        let max_value_str = U96_MAX.to_string();
        let result = CairoU96::from_str(&max_value_str).unwrap();
        assert_eq!(Felt::from(result), Felt::from(U96_MAX));
        let max_value_hex = format!("{U96_MAX:x}");
        let result_hex = CairoU96::from_str(&max_value_hex).unwrap();
        assert_eq!(Felt::from(result_hex), Felt::from(U96_MAX));
    }

    #[test_case("" ; "empty string")]
    #[test_case("not_a_number" ; "invalid string")]
    fn test_invalid_input(input: &str) {
        assert!(matches!(
            CairoU96::from_str(input),
            Err(ParseCairoU96Error::InvalidString(_))
        ));
    }

    #[test_case("0", 0_u128 ; "zero conversion")]
    #[test_case("123", 123_u128 ; "small number conversion")]
    #[test_case("1000000", 1_000_000_u128 ; "million conversion")]
    #[test_case("ff", 255_u128 ; "hex conversion")]
    #[test_case(&U96_MAX.to_string(), U96_MAX ; "max value conversion")]
    fn test_felt_conversion(input: &str, expected: u128) {
        let cairo_u96 = CairoU96::from_str(input).unwrap();
        assert_eq!(Felt::from(cairo_u96), Felt::from(expected));
    }
}
