use anyhow::{Context, Result};
use serde::Deserialize;
use starknet_types_core::felt::Felt;
use std::str::FromStr;

use crate::starknet_commands::multicall::contract_registry::ContractRegistry;

const ID_PREFIX: char = '@';

#[derive(Deserialize, Debug, Clone)]
pub struct ContractAddressIdentifier(String);

impl ContractAddressIdentifier {
    pub fn as_felt(&self) -> Result<Felt> {
        Felt::from_str(&self.0)
            .context("Failed to parse contract address: expected a hex or decimal string")
    }

    pub fn as_id(&self, contracts: &ContractRegistry) -> Result<Felt> {
        match self.0.strip_prefix(ID_PREFIX) {
            Some(id) => contracts
                .get_address_by_id(id)
                .context(format!("Failed to find contract address for id: {id}")),
            None => self.as_felt(),
        }
    }
}

impl std::str::FromStr for ContractAddressIdentifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ContractAddressIdentifier(s.to_owned()))
    }
}
