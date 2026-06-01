use anyhow::{Context, Result};
use starknet_types_core::felt::Felt;
use std::str::FromStr;

pub fn felt_from_string(data: &str) -> Result<Felt> {
    Felt::from_str(data).with_context(|| format!("Failed to parse `{data}` to felt"))
}
