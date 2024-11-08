use cairo_serde_macros::{CairoDeserialize, CairoSerialize};
use starknet::core::types::FromStrError;
use starknet_types_core::felt::Felt;
use std::str::FromStr;

#[derive(CairoDeserialize, CairoSerialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CairoBytes31 {
    inner: Felt,
}

impl CairoBytes31 {
    pub const MAX: CairoBytes31 = CairoBytes31 {
        inner: Felt::from_hex_unchecked(
            "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        ),
    };
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ParseBytes31Error {
    #[error("Failed to parse as Cairo type")]
    CairoFromStrError, // `FromStrError` thrown on unsuccessful Felt parsing is useless. We cannot write anything beyond that
    #[error("Number is too large to fit in 31 bytes")]
    Overflow,
}

impl From<FromStrError> for ParseBytes31Error {
    fn from(_value: FromStrError) -> Self {
        ParseBytes31Error::CairoFromStrError
    }
}

impl FromStr for CairoBytes31 {
    type Err = ParseBytes31Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let inner = input.parse::<Felt>()?;

        if inner > CairoBytes31::MAX.inner {
            return Err(ParseBytes31Error::Overflow);
        }

        Ok(CairoBytes31 { inner })
    }
}

impl From<CairoBytes31> for Felt {
    fn from(value: CairoBytes31) -> Self {
        value.inner
    }
}
