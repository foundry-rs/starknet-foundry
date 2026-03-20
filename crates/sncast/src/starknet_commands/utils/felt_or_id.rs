use anyhow::{Context, Result};
use serde::Deserialize;
use starknet_types_core::felt::Felt;
use std::str::FromStr;

const ID_PREFIX: char = '@';

#[derive(Deserialize, Debug, Clone)]
pub struct FeltOrId(String);

impl FeltOrId {
    pub fn new(s: String) -> Self {
        FeltOrId(s)
    }

    pub fn try_into_felt(&self) -> Result<Felt> {
        Felt::from_str(&self.0)
            .context("Failed to parse contract address: expected a hex or decimal string")
    }

    pub fn as_id(&self) -> Option<&str> {
        self.0.strip_prefix(ID_PREFIX)
    }
}

impl std::str::FromStr for FeltOrId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(FeltOrId(s.to_owned()))
    }
}
