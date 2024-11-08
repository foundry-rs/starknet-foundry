use crate::FromConv;
use cairo_serde_macros::CairoSerialize;
use conversions::from_thru_felt252;
use serde::{Deserialize, Serialize, Serializer};
use starknet_api::core::{ClassHash, ContractAddress};
use starknet_types_core::felt::Felt as Felt252;
use std::fmt;
use std::fmt::{Formatter, LowerHex};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, CairoSerialize)]
pub struct PaddedFelt(pub Felt252);

impl FromConv<Felt252> for PaddedFelt {
    fn from_(value: Felt252) -> Self {
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
        fmt::LowerHex::fmt(&self.0, f)
    }
}

from_thru_felt252!(ClassHash, PaddedFelt);
from_thru_felt252!(ContractAddress, PaddedFelt);
