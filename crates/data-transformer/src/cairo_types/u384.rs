use super::helpers::{ParseRadixError, RadixInput};
use cairo_serde_macros::{CairoDeserialize, CairoSerialize};
use num_bigint::BigUint;
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[expect(clippy::struct_field_names)]
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
        fn to_u128(slice: &[u8]) -> u128 {
            let mut padded = [0u8; 16];
            padded[4..].copy_from_slice(slice);
            u128::from_be_bytes(padded)
        }

        Self {
            limb_0: to_u128(&bytes[36..48]),
            limb_1: to_u128(&bytes[24..36]),
            limb_2: to_u128(&bytes[12..24]),
            limb_3: to_u128(&bytes[0..12]),
        }
    }

    #[must_use]
    pub fn to_be_bytes(&self) -> [u8; 48] {
        let mut result = [0; 48];

        result[36..48].copy_from_slice(&self.limb_0.to_be_bytes());
        result[24..36].copy_from_slice(&self.limb_1.to_be_bytes());
        result[12..24].copy_from_slice(&self.limb_2.to_be_bytes());
        result[0..12].copy_from_slice(&self.limb_3.to_be_bytes());

        result
    }
}

impl Display for CairoU384 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let number = BigUint::from_bytes_be(&self.to_be_bytes());
        write!(f, "{number}")
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
