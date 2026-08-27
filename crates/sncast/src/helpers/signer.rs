use anyhow::{Result, bail};
use camino::Utf8PathBuf;
use starknet_rust::signers::DerivationPath;

/// Represents the source of the signer for account operations
#[derive(Debug, Clone, Default)]
pub enum SignerSource {
    /// Create a native account backed by an encrypted keystore.
    Keystore(Utf8PathBuf),
    /// Use a Ledger device with the given derivation path
    Ledger(DerivationPath),
    /// Use the accounts file (default)
    #[default]
    AccountsFile,
}

impl SignerSource {
    pub fn new(ledger_path: Option<DerivationPath>, keystore: Option<Utf8PathBuf>) -> Result<Self> {
        match (ledger_path, keystore) {
            (Some(path), None) => Ok(Self::Ledger(path)),
            (None, Some(path)) => Ok(Self::Keystore(path)),
            (None, None) => Ok(Self::AccountsFile),
            (Some(_), Some(_)) => bail!("keystore and Ledger cannot be used together"),
        }
    }
}
