use super::helpers::{ParseRadixError, RadixInput};
use cairo_serde_macros::{CairoDeserialize, CairoSerialize};
use conversions;
use num_bigint::BigUint;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;
use thiserror;

#[derive(CairoDeserialize, CairoSerialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CairoU256 {
    low: u128,
    high: u128,
}

impl CairoU256 {
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        // Takes slice without explicit size because of cheatnet's specific usages (See Issue #2575)
        Self {
            low: u128::from_be_bytes(bytes[16..32].try_into().unwrap()),
            high: u128::from_be_bytes(bytes[0..16].try_into().unwrap()),
        }
    }

    #[must_use]
    pub fn to_be_bytes(&self) -> [u8; 32] {
        let mut result = [0; 32];

        result[16..].copy_from_slice(&self.low.to_be_bytes());
        result[..16].copy_from_slice(&self.high.to_be_bytes());

        result
    }
}

impl Display for CairoU256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let number = BigUint::from_bytes_be(&self.to_be_bytes());
        write!(f, "{number}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ParseCairoU256Error {
    #[error(transparent)]
    InvalidString(#[from] ParseRadixError),

    #[error("Number is too large to fit in 32 bytes")]
    Overflow,
}

impl FromStr for CairoU256 {
    type Err = ParseCairoU256Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let number: BigUint = RadixInput::try_from(input.as_bytes())?.try_into()?;
        let bytes = number.to_bytes_be();

        if bytes.len() > 32 {
            return Err(ParseCairoU256Error::Overflow);
        }

        let mut result = [0u8; 32];
        let start = 32 - bytes.len();
        result[start..].copy_from_slice(&bytes);

        Ok(CairoU256::from_bytes(&result))
    }
}

// Testing method: compare limbs against `Serde::serialize` output in Cairo
#[cfg(test)]
mod tests {
    use super::CairoU256;
    use test_case::test_case;

    const BIG_NUMBER_HEX: &str =
        "0xde0945c2474d9b33333123e53e37a93f5de4ba0adbf4c0a38afd2afd7d357f2c";
    const BIG_NUMBER_DEC: &str =
        "100429835467304823721949931582394957675800948774630560463857421711344858922796";

    const BIG_NUMBER_BYTES: [u8; 32] = [
        222, 9, 69, 194, 71, 77, 155, 51, 51, 49, 35, 229, 62, 55, 169, 63, 93, 228, 186, 10, 219,
        244, 192, 163, 138, 253, 42, 253, 125, 53, 127, 44,
    ];

    const TOO_BIG_NUMBER_BYTES: [u8; 48] = [
        222, 9, 69, 194, 71, 77, 155, 51, 51, 49, 35, 229, 62, 55, 169, 63, 93, 228, 186, 10, 219,
        244, 192, 163, 138, 253, 42, 253, 125, 53, 127, 44, 21, 37, 21, 37, 21, 37, 21, 37, 21, 37,
        21, 37, 21, 37, 21, 37,
    ];

    const BIG_NUMBER_LIMBS: [u128; 2] = [
        124_805_820_680_284_125_994_760_982_863_763_832_620,
        295_136_760_614_571_572_862_546_075_274_463_127_871,
    ];

    #[test_case(&[0; 32], [0, 0] ; "zeros")]
    #[test_case(&BIG_NUMBER_BYTES[..], BIG_NUMBER_LIMBS; "big")]
    fn test_happy_case_from_bytes(bytes: &[u8], expected_limbs: [u128; 2]) {
        let result = CairoU256::from_bytes(bytes);

        assert_eq!([result.low, result.high], expected_limbs);
    }

    #[should_panic(expected = "range end index 32 out of range for slice of length 4")]
    #[test]
    fn test_from_bytes_input_too_short() {
        let _result = CairoU256::from_bytes(&[2, 1, 3, 7]);
    }

    #[test]
    fn test_happy_case_from_bytes_longer_input() {
        let result = CairoU256::from_bytes(&TOO_BIG_NUMBER_BYTES);
        assert_eq!([result.low, result.high], BIG_NUMBER_LIMBS);
    }

    #[test_case("0x0", [0, 0] ; "zero_hex")]
    #[test_case("0", [0, 0] ; "zero_dec")]
    #[test_case("0x237abc", [2_325_180, 0] ; "small_hex")]
    #[test_case("237abc", [2_325_180, 0] ; "small_hex_no_leading_0x")]
    #[test_case("2325180", [2_325_180, 0] ; "small_dec")]
    #[test_case(BIG_NUMBER_HEX, BIG_NUMBER_LIMBS; "big_hex")]
    #[test_case(BIG_NUMBER_DEC, BIG_NUMBER_LIMBS; "big_dec")]
    fn test_happy_case_from_str(encoded: &str, expected_limbs: [u128; 2]) -> anyhow::Result<()> {
        let result: CairoU256 = encoded.parse()?;

        assert_eq!([result.low, result.high], expected_limbs);

        Ok(())
    }

    #[test_case([0, 0], "0"; "zero")]
    #[test_case([2_325_180, 0], "2325180"; "small")]
    #[test_case(BIG_NUMBER_LIMBS, BIG_NUMBER_DEC; "big")]
    fn test_display(limbs: [u128; 2], expected: &str) {
        let result = CairoU256 {
            low: limbs[0],
            high: limbs[1],
        };
        assert_eq!(result.to_string(), expected);
    }
}
