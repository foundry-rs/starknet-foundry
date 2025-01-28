use num_bigint::BigUint;
use starknet_types_core::felt::{Felt, NonZeroFelt};

#[cfg(test)]
mod tests_u384 {
    use super::*;

    #[test]
    fn test_u384_small_value() {
        let felt = Felt::from(42_u8);
        let _non_zero_felt = NonZeroFelt::try_from(felt).unwrap();
        let result = BigUint::from(42_u128);

        assert!(result.bits() <= 384);
    }

    #[test]
    fn test_u384_max_value() {
        // Precise 2^383 - 1 calculation to ensure exact 383-bit max value
        let max_u384 = BigUint::from(2_u128).pow(383) - BigUint::from(1_u128);

        // Explicit bit verification
        assert!(max_u384.bits() <= 384, "Value exceeds 384 bits");

        // Print out various representations
        println!("Decimal: {}", max_u384);
        println!("Hex direct: {:x}", max_u384);

        // Try direct Felt construction methods
        let felt = Felt::from(max_u384.clone());

        // Non-zero validation
        let _non_zero_felt = NonZeroFelt::try_from(felt).expect("Failed to create non-zero felt");
    }

    #[test]
    fn test_u384_too_large() {
        // Value explicitly larger than 2^384
        let large_felt = BigUint::from(2_u128).pow(384);

        // Verify it's genuinely larger than 384 bits
        assert!(large_felt.bits() > 384, "Value must exceed 384 bits");

        // Expect conversion to fail
        let conversion_result = Felt::from_dec_str(&large_felt.to_string());

        assert!(
            conversion_result.is_err(),
            "Conversion should fail for values exceeding 384 bits"
        );
    }
}
