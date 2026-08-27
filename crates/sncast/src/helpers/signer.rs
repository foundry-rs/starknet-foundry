use anyhow::{Result, bail};
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use starknet_rust::signers::DerivationPath;
use starknet_types_core::felt::Felt;

/// Represents the type of signer stored in the accounts file
// Uses `untagged` + `flatten` for backward compatibility with the existing accounts file format.
// Downside: deserialization errors are less descriptive than with tagged variants,
// and field name collisions across variants would silently misbehave.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum SignerType {
    Local { private_key: Felt },
    Ledger { ledger_path: DerivationPath },
}

impl SignerType {
    #[must_use]
    pub fn private_key(&self) -> Option<Felt> {
        match self {
            SignerType::Local { private_key } => Some(*private_key),
            SignerType::Ledger { .. } => None,
        }
    }

    #[must_use]
    pub fn ledger_path(&self) -> Option<&DerivationPath> {
        match self {
            SignerType::Ledger { ledger_path } => Some(ledger_path),
            SignerType::Local { .. } => None,
        }
    }
}

/// Represents the source of the signer for account operations
#[derive(Debug, Clone, Default)]
pub enum SignerSource {
    /// Use a keystore file at the given path
    Keystore(Utf8PathBuf),
    /// Use a Ledger device with the given derivation path
    Ledger(DerivationPath),
    /// Use the accounts file (default)
    #[default]
    AccountsFile,
}

impl SignerSource {
    pub fn new(keystore: Option<Utf8PathBuf>, ledger_path: Option<DerivationPath>) -> Result<Self> {
        match (keystore, ledger_path) {
            (Some(path), None) => Ok(SignerSource::Keystore(path)),
            (None, Some(path)) => Ok(SignerSource::Ledger(path)),
            (None, None) => Ok(SignerSource::AccountsFile),
            (Some(_), Some(_)) => {
                bail!("keystore and ledger cannot be used together")
            }
        }
    }
}
