use std::str::FromStr;

use starknet_rust::signers::DerivationPath;
use thiserror::Error;

const EIP_2645_LEVELS: usize = 6;
const EIP_2645_PURPOSE: u32 = 0x8000_0a55;
const HARDENED_BIT: u32 = 1 << 31;

#[derive(Debug, Error)]
pub enum DerivationPathError {
    #[error("invalid BIP-32 derivation path: {0}")]
    InvalidBip32(String),

    #[error("EIP-2645 paths must have {EIP_2645_LEVELS} levels")]
    InvalidLength,

    #[error("EIP-2645 paths must start with `m/2645'/`")]
    InvalidPurpose,

    #[error("the `{level}` level of an EIP-2645 path must be hardened")]
    Unhardened { level: &'static str },
}

/// Parses the canonical numeric EIP-2645 representation stored in an accounts file.
pub fn parse_derivation_path(value: &str) -> Result<DerivationPath, DerivationPathError> {
    let path = DerivationPath::from_str(value)
        .map_err(|error| DerivationPathError::InvalidBip32(error.to_string()))?;
    validate_derivation_path(&path)?;
    Ok(path)
}

pub fn validate_derivation_path(path: &DerivationPath) -> Result<(), DerivationPathError> {
    if path.len() != EIP_2645_LEVELS {
        return Err(DerivationPathError::InvalidLength);
    }

    let levels = path.iter().copied().collect::<Vec<_>>();
    if levels[0] != EIP_2645_PURPOSE {
        return Err(DerivationPathError::InvalidPurpose);
    }

    for (index, level) in ["layer", "application", "eth_address_1", "eth_address_2"]
        .into_iter()
        .enumerate()
    {
        if levels[index + 1] & HARDENED_BIT == 0 {
            return Err(DerivationPathError::Unhardened { level });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_eip_2645_path() {
        assert!(parse_derivation_path("m/2645'/1195502025'/355113700'/0'/0'/0").is_ok());
    }

    #[test]
    fn rejects_general_bip_32_path() {
        assert!(parse_derivation_path("m/44'/60'/0'/0/0").is_err());
    }
}
