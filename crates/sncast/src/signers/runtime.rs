use std::fmt::{Debug, Formatter};

use async_trait::async_trait;
use starknet_rust::core::crypto::Signature;
use starknet_rust::signers::{
    LedgerSigner, LocalWallet, Signer, SignerInteractivityContext, SigningKey, VerifyingKey,
};
use starknet_types_core::felt::Felt;

use crate::helpers::ledger::{self, SncastLedgerTransport};
use crate::response::ui::UI;
use crate::signers::error::KeystoreError;
use crate::signers::spec::PrivateKeySource;
use crate::signers::{
    KeystoreSpec, PrivateKeySpec, SignerError, SignerKind, SignerOperationError, SignerSpec,
    keystore_password,
};

pub enum RuntimeSigner {
    LocalWallet {
        signer: LocalWallet,
        source: PrivateKeySource,
    },
    Ledger(LedgerSigner<SncastLedgerTransport>),
}

impl RuntimeSigner {
    pub async fn from_spec(spec: SignerSpec, ui: &UI) -> Result<Self, SignerError> {
        match spec {
            SignerSpec::PrivateKey(spec) => Ok(Self::from(spec)),
            SignerSpec::Keystore(spec) => Self::try_from(spec),
            SignerSpec::Ledger(spec) => {
                ledger::create_ledger_signer(spec.derivation_path(), ui, false)
                    .await
                    .map(Self::from)
                    .map_err(Into::into)
            }
        }
    }

    #[must_use]
    pub fn from_private_key(private_key: Felt, source: PrivateKeySource) -> Self {
        let key = SigningKey::from_secret_scalar(private_key);
        let signer = LocalWallet::from_signing_key(key);
        Self::LocalWallet { signer, source }
    }

    #[must_use]
    pub fn kind(&self) -> SignerKind {
        match self {
            Self::LocalWallet { source, .. } => SignerKind::from(*source),
            Self::Ledger { .. } => SignerKind::Ledger,
        }
    }
}

impl From<PrivateKeySpec> for RuntimeSigner {
    fn from(spec: PrivateKeySpec) -> Self {
        Self::from_private_key(spec.private_key(), PrivateKeySource::PrivateKey)
    }
}

impl TryFrom<KeystoreSpec> for RuntimeSigner {
    type Error = SignerError;

    fn try_from(spec: KeystoreSpec) -> Result<Self, Self::Error> {
        let password = keystore_password(&spec)?;
        let key = SigningKey::from_keystore(spec.path(), &password).map_err(|source| {
            KeystoreError::Decrypt {
                path: spec.path().to_owned(),
                source,
            }
        })?;
        let wallet = LocalWallet::from_signing_key(key);
        Ok(RuntimeSigner::LocalWallet {
            signer: wallet,
            source: PrivateKeySource::Keystore,
        })
    }
}

impl From<LedgerSigner<SncastLedgerTransport>> for RuntimeSigner {
    fn from(signer: LedgerSigner<SncastLedgerTransport>) -> Self {
        Self::Ledger(signer)
    }
}

impl Debug for RuntimeSigner {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeSigner")
            .field("kind", &self.kind())
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl Signer for RuntimeSigner {
    type GetPublicKeyError = SignerError;
    type SignError = SignerError;

    async fn get_public_key(&self) -> Result<VerifyingKey, Self::GetPublicKeyError> {
        match self {
            Self::LocalWallet { signer, source } => get_public_key(signer, (*source).into()).await,
            Self::Ledger(signer) => get_public_key(signer, SignerKind::Ledger).await,
        }
    }

    async fn sign_hash(&self, hash: &Felt) -> Result<Signature, Self::SignError> {
        match self {
            Self::LocalWallet { signer, source } => sign_hash(signer, hash, (*source).into()).await,
            Self::Ledger(signer) => sign_hash(signer, hash, SignerKind::Ledger).await,
        }
    }

    fn is_interactive(&self, context: SignerInteractivityContext<'_>) -> bool {
        match self {
            Self::LocalWallet { signer, .. } => signer.is_interactive(context),
            Self::Ledger(signer) => signer.is_interactive(context),
        }
    }
}

async fn get_public_key<S>(signer: &S, kind: SignerKind) -> Result<VerifyingKey, SignerError>
where
    S: Signer,
    S::GetPublicKeyError: 'static,
{
    signer.get_public_key().await.map_err(|source| {
        SignerOperationError::GetPublicKey {
            kind,
            source: anyhow::Error::from(source),
        }
        .into()
    })
}

async fn sign_hash<S>(signer: &S, hash: &Felt, kind: SignerKind) -> Result<Signature, SignerError>
where
    S: Signer,
    S::SignError: 'static,
{
    signer.sign_hash(hash).await.map_err(|source| {
        SignerOperationError::SignHash {
            kind,
            source: anyhow::Error::from(source),
        }
        .into()
    })
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use starknet_rust::signers::SigningKey;
    use thiserror::Error;

    use super::*;

    #[derive(Debug, Error)]
    #[error("signer failure")]
    struct TestSignerError;

    struct FailingSigner;

    #[async_trait]
    impl Signer for FailingSigner {
        type GetPublicKeyError = TestSignerError;
        type SignError = TestSignerError;

        async fn get_public_key(&self) -> Result<VerifyingKey, Self::GetPublicKeyError> {
            Err(TestSignerError)
        }

        async fn sign_hash(&self, _hash: &Felt) -> Result<Signature, Self::SignError> {
            Err(TestSignerError)
        }

        fn is_interactive(&self, _context: SignerInteractivityContext<'_>) -> bool {
            false
        }
    }

    #[tokio::test]
    async fn delegates_successful_operations_to_local_wallet() {
        let key = SigningKey::from_secret_scalar(Felt::ONE);
        let expected_public_key = key.verifying_key();
        let signer =
            RuntimeSigner::from_private_key(key.secret_scalar(), PrivateKeySource::PrivateKey);

        assert_eq!(
            signer.get_public_key().await.unwrap().scalar(),
            expected_public_key.scalar()
        );
        assert!(signer.sign_hash(&Felt::TWO).await.is_ok());
        assert_eq!(signer.kind(), SignerKind::PrivateKey);
    }

    #[tokio::test]
    async fn wraps_public_key_errors_with_signer_kind_and_operation() {
        let error = get_public_key(&FailingSigner, SignerKind::Keystore)
            .await
            .unwrap_err();

        match error {
            SignerError::Operation(SignerOperationError::GetPublicKey { kind, source }) => {
                assert_eq!(kind, SignerKind::Keystore);
                assert_eq!(source.to_string(), "signer failure");
            }
            error => panic!("expected public-key operation error, got {error:?}"),
        }
    }

    #[tokio::test]
    async fn wraps_signing_errors_with_signer_kind_and_operation() {
        let error = sign_hash(&FailingSigner, &Felt::TWO, SignerKind::Ledger)
            .await
            .unwrap_err();

        match error {
            SignerError::Operation(SignerOperationError::SignHash { kind, source }) => {
                assert_eq!(kind, SignerKind::Ledger);
                assert_eq!(source.to_string(), "signer failure");
            }
            error => panic!("expected signing operation error, got {error:?}"),
        }
    }
}
