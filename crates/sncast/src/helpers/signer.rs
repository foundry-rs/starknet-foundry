use anyhow::{Result, bail};
use camino::Utf8PathBuf;
use starknet_rust::signers::DerivationPath;

/// Represents the source of the signer for account operations
#[derive(Debug, Clone, Default)]
pub enum SignerSource {
    /// Use a keystore file at the given path
    Keystore(Utf8PathBuf),
    /// Create a native account backed by an encrypted keystore.
    NativeKeystore(Utf8PathBuf),
    /// Use a Ledger device with the given derivation path
    Ledger(DerivationPath),
    /// Use the accounts file (default)
    #[default]
    AccountsFile,
}

impl SignerSource {
    pub fn new(
        keystore: Option<Utf8PathBuf>,
        ledger_path: Option<DerivationPath>,
        native_keystore: Option<Utf8PathBuf>,
    ) -> Result<Self> {
        match (keystore, ledger_path, native_keystore) {
            (Some(path), None, None) => Ok(Self::Keystore(path)),
            (None, Some(path), None) => Ok(Self::Ledger(path)),
            (None, None, Some(path)) => Ok(Self::NativeKeystore(path)),
            (None, None, None) => Ok(Self::AccountsFile),
            _ => bail!("legacy keystore, native keystore, and Ledger cannot be used together"),
        }
    }
}
