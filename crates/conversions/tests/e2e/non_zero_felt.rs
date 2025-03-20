#[cfg(test)]
mod tests_non_zero_felt {
    use std::num::{NonZeroU64, NonZeroU128};

    use conversions::FromConv;
    use starknet_types_core::felt::{Felt, NonZeroFelt};

    #[test]
    fn test_happy_case() {
        let non_zero_felt = NonZeroFelt::try_from(Felt::from(1_u8)).unwrap();

        assert_eq!(
            non_zero_felt,
            NonZeroFelt::from_(NonZeroU64::new(1).unwrap())
        );
        assert_eq!(
            non_zero_felt,
            NonZeroFelt::from_(NonZeroU128::new(1).unwrap())
        );
    }
}
