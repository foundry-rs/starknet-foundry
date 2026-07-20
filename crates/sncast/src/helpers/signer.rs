use crate::helpers::ledger;
use crate::response::ui::UI;
use anyhow::{Result, bail};
use camino::Utf8PathBuf;
use serde::ser::Error as SerError;
use serde::{Deserialize, Serialize, Serializer};
use serde_with::skip_serializing_none;
use starknet_rust::{
    accounts::SingleOwnerAccount,
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::{DerivationPath, LedgerSigner, LocalWallet, SigningKey},
};
use starknet_types_core::felt::Felt;

/// Signer type representation used in the accounts file.
/// For internal purposes, it should be converted to [`SignerType`].
#[skip_serializing_none]
#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
struct SignerTypeParams {
    private_key: Option<Felt>,
    ledger_path: Option<DerivationPath>,
}

/// Internal representation of a signer type.
/// [`SignerType::Ambiguous`] corresponds to zero or more than one signer types being specified.
#[derive(Clone, Debug, Deserialize)]
#[serde(from = "SignerTypeParams")]
pub enum SignerType {
    Local { private_key: Felt },
    Ledger { ledger_path: DerivationPath },
    Ambiguous,
}

impl From<SignerTypeParams> for SignerType {
    fn from(params: SignerTypeParams) -> Self {
        match (params.private_key, params.ledger_path) {
            (Some(private_key), None) => Self::Local { private_key },
            (None, Some(ledger_path)) => Self::Ledger { ledger_path },
            _ => Self::Ambiguous,
        }
    }
}

impl Serialize for SignerType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let raw = match self {
            SignerType::Local { private_key } => SignerTypeParams {
                private_key: Some(*private_key),
                ..Default::default()
            },
            SignerType::Ledger { ledger_path } => SignerTypeParams {
                ledger_path: Some(ledger_path.clone()),
                ..Default::default()
            },
            SignerType::Ambiguous => {
                return Err(S::Error::custom("cannot serialize ambiguous signer"));
            }
        };
        raw.serialize(serializer)
    }
}

impl SignerType {
    #[must_use]
    pub fn private_key(&self) -> Option<Felt> {
        match self {
            SignerType::Local { private_key } => Some(*private_key),
            SignerType::Ledger { .. } | SignerType::Ambiguous => None,
        }
    }

    #[must_use]
    pub fn ledger_path(&self) -> Option<&DerivationPath> {
        match self {
            SignerType::Ledger { ledger_path } => Some(ledger_path),
            SignerType::Local { .. } | SignerType::Ambiguous => None,
        }
    }
}

#[derive(Debug)]
pub enum SignerBackend {
    Local(LocalWallet),
    Ledger(LedgerSigner<ledger::SncastLedgerTransport>),
}

pub async fn build_signer(
    signer_type: &SignerType,
    ui: &UI,
    // TODO: unused in non-ledger context, consider refactor
    print_ledger_message: bool,
) -> Result<SignerBackend> {
    match signer_type {
        SignerType::Local { private_key } => Ok(SignerBackend::Local(LocalWallet::from(
            SigningKey::from_secret_scalar(*private_key),
        ))),
        SignerType::Ledger { ledger_path } => {
            let signer =
                ledger::create_ledger_signer(ledger_path, ui, print_ledger_message).await?;
            Ok(SignerBackend::Ledger(signer))
        }
        SignerType::Ambiguous => bail!("only one of `private_key`, `ledger_path` may be specified"),
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
