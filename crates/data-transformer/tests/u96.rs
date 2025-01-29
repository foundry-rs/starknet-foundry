#[cfg(test)]
mod tests_cairo_u96 {
    use data_transformer::cairo_types::{CairoU96, ParseCairoU96Error};
    use starknet_types_core::felt::Felt;
    use std::str::FromStr;

    const U96_MAX: u128 = (2u128 << 96) - 1;

    #[test]
    fn test_from_str_decimal() {
        let zero = CairoU96::from_str("0").unwrap();
        let small = CairoU96::from_str("123").unwrap();
        let large = CairoU96::from_str("1000000").unwrap();

        assert_eq!(Felt::from(zero), Felt::from(0_u128));
        assert_eq!(Felt::from(small), Felt::from(123_u128));
        assert_eq!(Felt::from(large), Felt::from(1000000_u128));
    }

    #[test]
    fn test_from_str_hex() {
        let zero = CairoU96::from_str("0").unwrap();
        let small_hex = CairoU96::from_str("ff").unwrap();
        let large_hex = CairoU96::from_str("1234abcd").unwrap();

        assert_eq!(Felt::from(zero), Felt::from(0_u128));
        assert_eq!(Felt::from(small_hex), Felt::from(255_u128));
        assert_eq!(Felt::from(large_hex), Felt::from(0x1234abcd_u128));
    }

    #[test]
    fn test_from_str_max_value() {
        let max_value_str = U96_MAX.to_string();
        let result = CairoU96::from_str(&max_value_str).unwrap();
        assert_eq!(Felt::from(result), Felt::from(U96_MAX));
        let max_value_hex = format!("{:x}", U96_MAX);
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
            ("1000000", 1000000_u128),
            ("ff", 255_u128),
            (&U96_MAX.to_string(), U96_MAX),
        ];

        for (input, expected) in values {
            let cairo_u96 = CairoU96::from_str(input).unwrap();
            assert_eq!(Felt::from(cairo_u96), Felt::from(expected));
        }
    }
}
