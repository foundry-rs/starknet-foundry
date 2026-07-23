use crate::get_keystore_password;
use crate::helpers::constants::KEYSTORE_PASSWORD_ENV_VAR;
use crate::helpers::ledger;
use crate::response::ui::UI;
use anyhow::{Context, Result, bail};
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
    keystore_path: Option<Utf8PathBuf>,
}

/// Internal representation of a signer type.
/// [`SignerType::Ambiguous`] corresponds to zero or more than one signer types being specified.
#[derive(Clone, Debug, Deserialize)]
#[serde(from = "SignerTypeParams")]
pub enum SignerType {
    PrivateKey { private_key: Felt },
    Ledger { ledger_path: DerivationPath },
    Keystore { keystore_path: Utf8PathBuf },
    Ambiguous,
}

impl From<SignerTypeParams> for SignerType {
    fn from(params: SignerTypeParams) -> Self {
        match (params.private_key, params.ledger_path, params.keystore_path) {
            (Some(private_key), None, None) => Self::PrivateKey { private_key },
            (None, Some(ledger_path), None) => Self::Ledger { ledger_path },
            (None, None, Some(keystore_path)) => Self::Keystore { keystore_path },
            _ => Self::Ambiguous,
        }
    }
}

impl Serialize for SignerType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let raw = match self {
            SignerType::PrivateKey { private_key } => SignerTypeParams {
                private_key: Some(*private_key),
                ..Default::default()
            },
            SignerType::Ledger { ledger_path } => SignerTypeParams {
                ledger_path: Some(ledger_path.clone()),
                ..Default::default()
            },
            SignerType::Keystore { keystore_path } => SignerTypeParams {
                keystore_path: Some(keystore_path.clone()),
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
            SignerType::PrivateKey { private_key } => Some(*private_key),
            SignerType::Ledger { .. } | SignerType::Keystore { .. } | SignerType::Ambiguous => None,
        }
    }

    #[must_use]
    pub fn ledger_path(&self) -> Option<&DerivationPath> {
        match self {
            SignerType::Ledger { ledger_path } => Some(ledger_path),
            SignerType::PrivateKey { .. } | SignerType::Keystore { .. } | SignerType::Ambiguous => {
                None
            }
        }
    }

    #[must_use]
    pub fn keystore_path(&self) -> Option<&Utf8PathBuf> {
        match self {
            SignerType::Keystore { keystore_path } => Some(keystore_path),
            SignerType::PrivateKey { .. } | SignerType::Ledger { .. } | SignerType::Ambiguous => {
                None
            }
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
        SignerType::PrivateKey { private_key } => Ok(SignerBackend::Local(LocalWallet::from(
            SigningKey::from_secret_scalar(*private_key),
        ))),
        SignerType::Keystore { keystore_path } => {
            let password = get_keystore_password(KEYSTORE_PASSWORD_ENV_VAR)?;
            let key = SigningKey::from_keystore(keystore_path, password.as_str())
                .with_context(|| format!("Failed to decrypt keystore at {keystore_path}"))?;
            Ok(SignerBackend::Local(LocalWallet::from(key)))
        }
        SignerType::Ledger { ledger_path } => {
            let signer =
                ledger::create_ledger_signer(ledger_path, ui, print_ledger_message).await?;
            Ok(SignerBackend::Ledger(signer))
        }
        SignerType::Ambiguous => {
            bail!("only one of `private_key`, `ledger_path`, `keystore_path` may be specified")
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

/// Create-time strategy for the new account's signer:
/// Where the secret comes from and how it will be persisted.
/// This is a `account create`-only concept. [`SignerType`] is used otherwise.
#[derive(Debug, Clone, Default)]
pub enum CreateSignerStrategy {
    /// Use a keystore file at the given path (legacy format)
    // TODO: remove legacy global keystore logic
    LegacyKeystore(Utf8PathBuf),
    /// Use a keystore file at the given path
    RegistryKeystore(Utf8PathBuf),
    /// Use a Ledger device with the given derivation path
    Ledger(DerivationPath),
    /// Use the private key stored in the accounts file (default)
    // TODO: make non-default and deprecate this option
    #[default]
    PrivateKey,
}

impl CreateSignerStrategy {
    // TODO: remove legacy global keystore logic
    pub fn new(
        keystore: Option<Utf8PathBuf>,
        ledger_path: Option<DerivationPath>,
        legacy_keystore: Option<Utf8PathBuf>,
    ) -> Result<Self> {
        match (keystore, ledger_path, legacy_keystore) {
            // TODO: remove legacy global keystore logic
            (Some(_), _, Some(_)) => bail!(
                "`account create --keystore` / `--keystore-path` cannot be used together with global `--keystore`"
            ),
            (Some(path), None, None) => Ok(Self::RegistryKeystore(path)),
            (None, None, Some(path)) => Ok(Self::LegacyKeystore(path)),
            (None, Some(path), None) => Ok(Self::Ledger(path)),
            (None, None, None) => Ok(Self::PrivateKey),
            (Some(_), Some(_), None) | (_, Some(_), Some(_)) => {
                bail!("keystore and ledger cannot be used together")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountData;

    #[test]
    fn unknown_fields_captured_on_account_data() {
        let data: AccountData = serde_json::from_str(
            r#"{
                "public_key": "0x1",
                "address": "0x2",
                "private_key": "0x3",
                "typo_field": 1
            }"#,
        )
        .unwrap();
        assert!(data.unknown_fields.contains_key("typo_field"));
    }

    #[tokio::test]
    async fn build_signer_rejects_ambiguous() {
        let ui = UI::default();
        let err = build_signer(&SignerType::Ambiguous, &ui, false)
            .await
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "only one of `private_key`, `ledger_path` may be specified"
        );
    }

    #[test]
    fn serialize_ambiguous_hard_fails() {
        let err = serde_json::to_string(&SignerType::Ambiguous).unwrap_err();
        assert!(
            err.to_string()
                .contains("cannot serialize ambiguous signer"),
            "unexpected error: {err}"
        );
    }
}
