use crate as conversions; // Must be imported because of derive macros
use cairo_serde_macros::{CairoDeserialize, CairoSerialize};
use num_bigint::{BigUint, ParseBigIntError};
use std::str::FromStr;

#[derive(CairoDeserialize, CairoSerialize, Debug)]
pub struct CairoU512 {
    limb_0: u128,
    limb_1: u128,
    limb_2: u128,
    limb_3: u128,
}

impl CairoU512 {
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
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

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ParseCairoU512Error {
    #[error(transparent)]
    InvalidString(#[from] ParseBigIntError),
    #[error("Number is too large to fit in 64 bytes")]
    Overflow,
}

impl FromStr for CairoU512 {
    type Err = ParseCairoU512Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let bytes = input.parse::<BigUint>()?.to_bytes_be();

        if bytes.len() > 64 {
            return Err(ParseCairoU512Error::Overflow);
        }

        let mut result = [0u8; 64];
        let start = 64 - bytes.len();
        result[start..].copy_from_slice(&bytes);

        Ok(CairoU512::from_bytes(&result))
    }
}
