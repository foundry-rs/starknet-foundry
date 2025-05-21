use crate::FromConv;
use cairo_serde_macros::CairoSerialize;
use conversions::from_thru_felt;
use serde::{Deserialize, Serialize, Serializer};
use starknet_api::core::{ClassHash, ContractAddress};
use starknet_types_core::felt::Felt;
use std::fmt;
use std::fmt::{Display, Formatter, LowerHex};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, CairoSerialize)]
pub struct PaddedFelt(pub Felt);

impl FromConv<Felt> for PaddedFelt {
    fn from_(value: Felt) -> Self {
        Self(value)
    }
}

impl Serialize for PaddedFelt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:#064x}", &self.0))
    }
}

impl LowerHex for PaddedFelt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#064x}", self.0)
    }
}

impl Display for PaddedFelt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#064x}", self.0)
    }
}

from_thru_felt!(ClassHash, PaddedFelt);
from_thru_felt!(ContractAddress, PaddedFelt);
