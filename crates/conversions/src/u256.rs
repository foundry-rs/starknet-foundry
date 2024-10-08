use crate as conversions; // Must be imported because of derive macros
use cairo_serde_macros::{CairoDeserialize, CairoSerialize};
use num_bigint::{BigUint, ParseBigIntError};
use std::str::FromStr;

#[derive(CairoDeserialize, CairoSerialize, Debug)]
pub struct CairoU256 {
    low: u128,
    high: u128,
}

impl CairoU256 {
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
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

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ParseCairoU256Error {
    #[error(transparent)]
    InvalidString(#[from] ParseBigIntError),
    #[error("Number is too large to fit in 32 bytes")]
    Overflow,
}

impl FromStr for CairoU256 {
    type Err = ParseCairoU256Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let bytes = input.parse::<BigUint>()?.to_bytes_be();

        if bytes.len() > 32 {
            return Err(ParseCairoU256Error::Overflow);
        }

        let mut result = [0u8; 32];
        let start = 32 - bytes.len();
        result[start..].copy_from_slice(&bytes);

        Ok(CairoU256::from_bytes(&result))
    }
}
