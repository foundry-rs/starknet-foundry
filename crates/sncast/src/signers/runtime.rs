use std::fmt::{Debug, Formatter};

use async_trait::async_trait;
use starknet_rust::core::crypto::Signature;
use starknet_rust::signers::{
    LedgerSigner, LocalWallet, Signer, SignerInteractivityContext, VerifyingKey,
};
use starknet_types_core::felt::Felt;

use crate::helpers::ledger::SncastLedgerTransport;
use crate::signers::{SignerError, SignerKind};

pub enum RuntimeSigner {
    LocalWallet {
        signer: LocalWallet,
        kind: SignerKind,
    },
    Ledger {
        signer: LedgerSigner<SncastLedgerTransport>,
    },
}

impl RuntimeSigner {
    pub(crate) fn from_local_wallet(signer: LocalWallet, kind: SignerKind) -> Self {
        Self::LocalWallet { signer, kind }
    }

    pub(crate) fn from_ledger_signer(signer: LedgerSigner<SncastLedgerTransport>) -> Self {
        Self::Ledger { signer }
    }

    #[must_use]
    pub fn kind(&self) -> SignerKind {
        match self {
            Self::LocalWallet { kind, .. } => *kind,
            Self::Ledger { .. } => SignerKind::Ledger,
        }
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
            Self::LocalWallet { signer, kind } => get_public_key(signer, *kind).await,
            Self::Ledger { signer } => get_public_key(signer, SignerKind::Ledger).await,
        }
    }

    async fn sign_hash(&self, hash: &Felt) -> Result<Signature, Self::SignError> {
        match self {
            Self::LocalWallet { signer, kind } => sign_hash(signer, hash, *kind).await,
            Self::Ledger { signer } => sign_hash(signer, hash, SignerKind::Ledger).await,
        }
    }

    fn is_interactive(&self, context: SignerInteractivityContext<'_>) -> bool {
        match self {
            Self::LocalWallet { signer, .. } => signer.is_interactive(context),
            Self::Ledger { signer, .. } => signer.is_interactive(context),
        }
    }
}

async fn get_public_key<S>(signer: &S, kind: SignerKind) -> Result<VerifyingKey, SignerError>
where
    S: Signer + Send + Sync,
{
    signer
        .get_public_key()
        .await
        .map_err(|error| SignerError::Backend {
            kind,
            operation: "get public key",
            message: error.to_string(),
        })
}

async fn sign_hash<S>(signer: &S, hash: &Felt, kind: SignerKind) -> Result<Signature, SignerError>
where
    S: Signer + Send + Sync,
{
    signer
        .sign_hash(hash)
        .await
        .map_err(|error| SignerError::Backend {
            kind,
            operation: "sign hash",
            message: error.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use starknet_rust::signers::{LocalWallet, SigningKey};

    use super::*;

    #[tokio::test]
    async fn adapts_starknet_signers_to_one_error_type() {
        let key = SigningKey::from_secret_scalar(Felt::ONE);
        let expected_public_key = key.verifying_key();
        let signer = RuntimeSigner::from_local_wallet(
            LocalWallet::from_signing_key(key),
            SignerKind::PrivateKey,
        );

        assert_eq!(
            signer.get_public_key().await.unwrap().scalar(),
            expected_public_key.scalar()
        );
        assert!(signer.sign_hash(&Felt::TWO).await.is_ok());
        assert_eq!(signer.kind(), SignerKind::PrivateKey);
    }
}
