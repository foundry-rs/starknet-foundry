#[cfg(test)]
mod tests_non_zero_u128 {
    use conversions::TryFromConv;
    use starknet_types_core::felt::{Felt, NonZeroFelt};
    use std::num::NonZeroU128;

    #[test]
    fn test_happy_case() {
        let non_zero_u128 = NonZeroU128::new(1).unwrap();

        assert_eq!(
            non_zero_u128,
            NonZeroU128::try_from_(NonZeroFelt::try_from(Felt::from(1_u8)).unwrap()).unwrap()
        );
    }

    #[test]
    fn test_limit() {
        let felt = Felt::from_dec_str(&u128::MAX.to_string()).unwrap();
        let non_zero_felt = NonZeroFelt::try_from(felt).unwrap();

        let result = NonZeroU128::try_from_(non_zero_felt);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get(), u128::MAX);
    }

    #[test]
    fn test_felt_too_large() {
        let large_felt = Felt::from_dec_str("340282366920938463463374607431768211456").unwrap(); // 2^128
        let non_zero_felt = NonZeroFelt::try_from(large_felt).unwrap();

        let result = NonZeroU128::try_from_(non_zero_felt);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "felt was too large to fit in u128");
    }
}
