use super::helpers::{ParseRadixError, RadixInput};
use cairo_serde_macros::{CairoDeserialize, CairoSerialize};
use num_bigint::BigUint;
use std::str::FromStr;

#[derive(CairoDeserialize, CairoSerialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CairoU384 {
    limb_0: u128,
    limb_1: u128,
    limb_2: u128,
    limb_3: u128,
}

impl CairoU384 {
    #[must_use]
    pub fn from_bytes(bytes: &[u8; 48]) -> Self {
        Self {
            limb_0: u128::from_be_bytes(bytes[36..48].try_into().unwrap()),
            limb_1: u128::from_be_bytes(bytes[24..36].try_into().unwrap()),
            limb_2: u128::from_be_bytes(bytes[12..24].try_into().unwrap()),
            limb_3: u128::from_be_bytes(bytes[00..12].try_into().unwrap()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ParseCairoU384Error {
    #[error(transparent)]
    InvalidString(#[from] ParseRadixError),
    #[error("Number is too large to fit in 48 bytes")]
    Overflow,
}

impl FromStr for CairoU384 {
    type Err = ParseCairoU384Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let number: BigUint = RadixInput::try_from(input.as_bytes())?.try_into()?;
        let bytes = number.to_bytes_be();

        if bytes.len() > 48 {
            return Err(ParseCairoU384Error::Overflow);
        }

        let mut result = [0u8; 48];
        let start = 48 - bytes.len();
        result[start..].copy_from_slice(&bytes);

        Ok(CairoU384::from_bytes(&result))
    }
}
