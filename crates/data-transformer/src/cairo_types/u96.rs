use cairo_serde_macros::{CairoDeserialize, CairoSerialize};
use starknet_types_core::felt::Felt;
use std::fmt::Display;
use std::{
    fmt,
    num::{IntErrorKind, ParseIntError},
    str::FromStr,
};

#[derive(CairoSerialize, CairoDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CairoU96 {
    inner: u128,
}

impl Display for CairoU96 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

const MAX_VALUE: u128 = (2 << 96) - 1;

impl From<CairoU96> for Felt {
    fn from(value: CairoU96) -> Self {
        Felt::from(value.inner)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ParseCairoU96Error {
    #[error(transparent)]
    InvalidString(#[from] ParseIntError),
    #[error("Number is too large to fit in 24 bytes")]
    Overflow,
}

impl FromStr for CairoU96 {
    type Err = ParseCairoU96Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let is_hex = input.starts_with("0x") || input.contains(|c: char| c.is_alphabetic());

        let number = if is_hex {
            u128::from_str_radix(input, 16)
        } else {
            u128::from_str(input)
        }
        .map_err(|err| {
            if err.kind() == &IntErrorKind::PosOverflow {
                ParseCairoU96Error::Overflow
            } else {
                err.into()
            }
        })?;

        if number > MAX_VALUE {
            return Err(ParseCairoU96Error::Overflow);
        }

        Ok(Self { inner: number })
    }
}
