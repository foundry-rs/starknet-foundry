use super::helpers::{ParseRadixError, RadixInput};
use cairo_serde_macros::{CairoDeserialize, CairoSerialize};
use num_bigint::BigUint;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[allow(clippy::struct_field_names)]
#[derive(CairoDeserialize, CairoSerialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CairoU512 {
    limb_0: u128,
    limb_1: u128,
    limb_2: u128,
    limb_3: u128,
}

impl CairoU512 {
    #[must_use]
    pub fn from_bytes(bytes: &[u8; 64]) -> Self {
        Self {
            limb_0: u128::from_be_bytes(bytes[48..64].try_into().unwrap()),
            limb_1: u128::from_be_bytes(bytes[32..48].try_into().unwrap()),
            limb_2: u128::from_be_bytes(bytes[16..32].try_into().unwrap()),
            limb_3: u128::from_be_bytes(bytes[00..16].try_into().unwrap()),
        }
    }

    #[must_use]
    pub fn to_be_bytes(&self) -> [u8; 64] {
        let mut result = [0; 64];

        result[48..64].copy_from_slice(&self.limb_0.to_be_bytes());
        result[32..48].copy_from_slice(&self.limb_1.to_be_bytes());
        result[16..32].copy_from_slice(&self.limb_2.to_be_bytes());
        result[00..16].copy_from_slice(&self.limb_3.to_be_bytes());

        result
    }
}

impl Display for CairoU512 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let number = BigUint::from_bytes_be(&self.to_be_bytes());
        write!(f, "{number}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ParseCairoU512Error {
    #[error(transparent)]
    InvalidString(#[from] ParseRadixError),
    #[error("Number is too large to fit in 64 bytes")]
    Overflow,
}

impl FromStr for CairoU512 {
    type Err = ParseCairoU512Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let number: BigUint = RadixInput::try_from(input.as_bytes())?.try_into()?;
        let bytes = number.to_bytes_be();

        if bytes.len() > 64 {
            return Err(ParseCairoU512Error::Overflow);
        }

        let mut result = [0u8; 64];
        let start = 64 - bytes.len();
        result[start..].copy_from_slice(&bytes);

        Ok(CairoU512::from_bytes(&result))
    }
}

// Testing method: compare limbs against `Serde::serialize` output in Cairo
#[cfg(test)]
mod tests {
    use super::CairoU512;
    use test_case::test_case;

    const BIG_NUMBER_HEX: &str = "0xec6710e3f6607d8528d37b2b7110c1a65d6482a9bd5cf8d6fe0620ce8972c857960c53c1c06a94c104957f378fa4a3a080b84117d9d093466849643204da84e7";
    const BIG_NUMBER_DEC: &str = "12381408885777547607539348003833063591238452153837122447405738741626823601474822019420743833187140657614799860086984246159941269173037600465935986717263079";

    const BIG_NUMBER_BYTES: [u8; 64] = [
        236, 103, 16, 227, 246, 96, 125, 133, 40, 211, 123, 43, 113, 16, 193, 166, 93, 100, 130,
        169, 189, 92, 248, 214, 254, 6, 32, 206, 137, 114, 200, 87, 150, 12, 83, 193, 192, 106,
        148, 193, 4, 149, 127, 55, 143, 164, 163, 160, 128, 184, 65, 23, 217, 208, 147, 70, 104,
        73, 100, 50, 4, 218, 132, 231,
    ];

    const BIG_NUMBER_LIMBS: [u128; 4] = [
        171_097_886_328_722_014_390_365_673_661_673_407_719,
        199_448_205_720_622_237_702_144_553_019_244_848_032,
        124_140_083_455_263_661_715_169_373_816_437_786_711,
        314_232_956_161_265_744_462_671_566_462_772_101_542,
    ];

    #[test_case(&[0; 64], [0, 0, 0, 0] ; "zeros")]
    #[test_case(&BIG_NUMBER_BYTES, BIG_NUMBER_LIMBS; "big")]
    fn test_happy_case_from_bytes(bytes: &[u8; 64], expected_limbs: [u128; 4]) {
        let result = CairoU512::from_bytes(bytes);

        assert_eq!(
            [result.limb_0, result.limb_1, result.limb_2, result.limb_3],
            expected_limbs
        );
    }

    #[test_case("0x0", [0, 0, 0, 0] ; "zero_hex")]
    #[test_case("0", [0, 0, 0, 0] ; "zero_dec")]
    #[test_case("0x237abc", [2_325_180, 0, 0, 0] ; "small_hex")]
    #[test_case("237abc", [2_325_180, 0, 0, 0] ; "small_hex_no_leading_0x")]
    #[test_case("2325180", [2_325_180, 0, 0, 0] ; "small_dec")]
    #[test_case(BIG_NUMBER_HEX, BIG_NUMBER_LIMBS; "big_hex")]
    #[test_case(BIG_NUMBER_DEC, BIG_NUMBER_LIMBS; "big_dec")]
    fn test_happy_case_from_str(encoded: &str, expected_limbs: [u128; 4]) -> anyhow::Result<()> {
        let result: CairoU512 = encoded.parse()?;

        assert_eq!(
            [result.limb_0, result.limb_1, result.limb_2, result.limb_3],
            expected_limbs
        );

        Ok(())
    }

    #[test_case([0, 0, 0, 0], "0"; "zero")]
    #[test_case([2_325_180, 0, 0, 0], "2325180"; "small")]
    #[test_case(BIG_NUMBER_LIMBS, BIG_NUMBER_DEC; "big")]
    fn test_display(limbs: [u128; 4], expected: &str) {
        let number = CairoU512 {
            limb_0: limbs[0],
            limb_1: limbs[1],
            limb_2: limbs[2],
            limb_3: limbs[3],
        };

        assert_eq!(number.to_string(), expected);
    }
}
