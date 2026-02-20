use crate::helpers::ledger;
use anyhow::{Result, bail};
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use starknet_rust::{
    accounts::SingleOwnerAccount,
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::{DerivationPath, LedgerSigner, LocalWallet},
};
use starknet_types_core::felt::Felt;

/// Represents the type of signer stored in the accounts file
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

#[derive(Debug)]
/// Represents the `SingleOwnerAccount` variant with either `LocalWallet` or `LedgerSigner` as signer
pub enum AccountVariant<'a> {
    LocalWallet(SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, LocalWallet>),
    Ledger(
        SingleOwnerAccount<
            &'a JsonRpcClient<HttpTransport>,
            LedgerSigner<ledger::SncastLedgerTransport>,
        >,
    ),
}

impl AccountVariant<'_> {
    #[must_use]
    pub fn address(&self) -> Felt {
        use starknet_rust::accounts::Account;
        match self {
            AccountVariant::LocalWallet(account) => account.address(),
            AccountVariant::Ledger(account) => account.address(),
        }
    }

    #[must_use]
    pub fn chain_id(&self) -> Felt {
        use starknet_rust::accounts::Account;
        match self {
            AccountVariant::LocalWallet(account) => account.chain_id(),
            AccountVariant::Ledger(account) => account.chain_id(),
        }
    }
}

#[macro_export]
macro_rules! with_account {
    ($variant:expr, |$account:ident| $body:expr) => {
        match $variant {
            &$crate::AccountVariant::LocalWallet(ref $account) => $body,
            &$crate::AccountVariant::Ledger(ref $account) => $body,
        }
    };
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
    pub fn new(keystore: Option<Utf8PathBuf>, signer_type: Option<&SignerType>) -> Result<Self> {
        let ledger_path = signer_type.and_then(SignerType::ledger_path);
        match (keystore, ledger_path) {
            (Some(path), None) => Ok(SignerSource::Keystore(path)),
            (None, Some(path)) => Ok(SignerSource::Ledger(path.clone())),
            (None, None) => Ok(SignerSource::AccountsFile),
            (Some(_), Some(_)) => {
                bail!("keystore and ledger cannot be used together")
            }
        }
    }
}
