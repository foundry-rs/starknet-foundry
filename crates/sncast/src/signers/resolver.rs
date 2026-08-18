use std::sync::Arc;

use async_trait::async_trait;
use camino::{Utf8Path, Utf8PathBuf};
use starknet_rust::signers::{LocalWallet, Signer, SigningKey};
use starknet_types_core::felt::Felt;

use crate::accounts::AccountRepository;
use crate::helpers::ledger;
use crate::response::ui::UI;
use crate::signers::{
    KeystoreFile, RuntimeSigner, SignerError, SignerKind, SignerSpec, keystore_password,
};

pub struct SignerProviderContext<'a> {
    pub repository: &'a AccountRepository,
    pub ui: &'a UI,
}

#[async_trait]
pub trait SignerProvider: Send + Sync {
    fn kind(&self) -> SignerKind;

    async fn resolve(
        &self,
        spec: &SignerSpec,
        context: &SignerProviderContext<'_>,
    ) -> Result<RuntimeSigner, SignerError>;
}

pub struct SignerResolver {
    providers: Vec<Arc<dyn SignerProvider>>,
}

impl Default for SignerResolver {
    fn default() -> Self {
        Self::new(vec![
            Arc::new(PrivateKeySignerProvider),
            Arc::new(KeystoreSignerProvider),
            Arc::new(LedgerSignerProvider),
        ])
    }
}

impl SignerResolver {
    #[must_use]
    pub fn new(providers: Vec<Arc<dyn SignerProvider>>) -> Self {
        Self { providers }
    }

    pub async fn resolve(
        &self,
        spec: &SignerSpec,
        context: &SignerProviderContext<'_>,
    ) -> Result<RuntimeSigner, SignerError> {
        let kind = spec.kind();
        let provider = self
            .providers
            .iter()
            .find(|provider| provider.kind() == kind)
            .ok_or(SignerError::Unsupported { kind })?;
        provider.resolve(spec, context).await
    }

    pub async fn resolve_and_verify(
        &self,
        spec: &SignerSpec,
        expected_public_key: Felt,
        context: &SignerProviderContext<'_>,
    ) -> Result<RuntimeSigner, SignerError> {
        let signer = self.resolve(spec, context).await?;
        let actual = signer.get_public_key().await?.scalar();
        if actual != expected_public_key {
            return Err(SignerError::PublicKeyMismatch {
                kind: signer.kind(),
                expected: expected_public_key,
                actual,
            });
        }
        Ok(signer)
    }
}

struct PrivateKeySignerProvider;

#[async_trait]
impl SignerProvider for PrivateKeySignerProvider {
    fn kind(&self) -> SignerKind {
        SignerKind::PrivateKey
    }

    async fn resolve(
        &self,
        spec: &SignerSpec,
        _context: &SignerProviderContext<'_>,
    ) -> Result<RuntimeSigner, SignerError> {
        let SignerSpec::PrivateKey(spec) = spec else {
            return Err(SignerError::Unsupported { kind: spec.kind() });
        };
        let key = SigningKey::from_secret_scalar(spec.private_key());
        Ok(RuntimeSigner::from_local_wallet(
            LocalWallet::from_signing_key(key),
            self.kind(),
        ))
    }
}

pub struct KeystoreSignerProvider;

#[async_trait]
impl SignerProvider for KeystoreSignerProvider {
    fn kind(&self) -> SignerKind {
        SignerKind::Keystore
    }

    async fn resolve(
        &self,
        spec: &SignerSpec,
        context: &SignerProviderContext<'_>,
    ) -> Result<RuntimeSigner, SignerError> {
        let SignerSpec::Keystore(spec) = spec else {
            return Err(SignerError::Unsupported { kind: spec.kind() });
        };
        let path = resolve_keystore_path(context.repository.path(), spec.path());
        let password = keystore_password(spec)?;
        let key = KeystoreFile::decrypt(&path, &password)?;
        Ok(RuntimeSigner::from_local_wallet(
            LocalWallet::from_signing_key(key),
            self.kind(),
        ))
    }
}

struct LedgerSignerProvider;

#[async_trait]
impl SignerProvider for LedgerSignerProvider {
    fn kind(&self) -> SignerKind {
        SignerKind::Ledger
    }

    async fn resolve(
        &self,
        spec: &SignerSpec,
        context: &SignerProviderContext<'_>,
    ) -> Result<RuntimeSigner, SignerError> {
        let SignerSpec::Ledger(spec) = spec else {
            return Err(SignerError::Unsupported { kind: spec.kind() });
        };
        let signer = ledger::create_ledger_signer(spec.derivation_path(), context.ui, false)
            .await
            .map_err(|error| SignerError::Backend {
                kind: self.kind(),
                operation: "connect to device",
                message: error.to_string(),
            })?;
        Ok(RuntimeSigner::from_ledger_signer(signer))
    }
}

#[must_use]
pub fn resolve_keystore_path(accounts_file: &Utf8Path, path: &Utf8Path) -> Utf8PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        accounts_file
            .parent()
            .unwrap_or(Utf8Path::new("."))
            .join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signers::PrivateKeySpec;

    #[test]
    fn resolves_relative_keystore_paths_from_accounts_file() {
        assert_eq!(
            resolve_keystore_path(
                Utf8Path::new("config/accounts.json"),
                Utf8Path::new("keys/alice.json")
            ),
            Utf8PathBuf::from("config/keys/alice.json")
        );
    }

    #[tokio::test]
    async fn resolves_and_verifies_private_key_signer() {
        let ui = UI::default();
        let key = SigningKey::from_secret_scalar(Felt::ONE);
        let expected = key.verifying_key().scalar();
        let spec = SignerSpec::PrivateKey(PrivateKeySpec::new(Felt::ONE));
        let context = SignerProviderContext {
            repository: &AccountRepository::new(Utf8PathBuf::from("accounts.json")),
            ui: &ui,
        };

        let signer = SignerResolver::default()
            .resolve_and_verify(&spec, expected, &context)
            .await
            .unwrap();
        assert_eq!(signer.kind(), SignerKind::PrivateKey);
    }

    #[tokio::test]
    async fn rejects_public_key_mismatch() {
        let ui = UI::default();
        let spec = SignerSpec::PrivateKey(PrivateKeySpec::new(Felt::ONE));
        let context = SignerProviderContext {
            repository: &AccountRepository::new(Utf8PathBuf::from("accounts.json")),
            ui: &ui,
        };

        assert!(matches!(
            SignerResolver::default()
                .resolve_and_verify(&spec, Felt::TWO, &context)
                .await,
            Err(SignerError::PublicKeyMismatch { .. })
        ));
    }
}
