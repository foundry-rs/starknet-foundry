use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_trait::async_trait;
use starknet_rust::core::crypto::Signature;
use starknet_rust::signers::{Signer, SignerInteractivityContext, VerifyingKey};
use starknet_types_core::felt::Felt;

use crate::signers::backend::{SignerBackend, StarknetSignerBackend};
use crate::signers::{SignerError, SignerKind};

#[derive(Clone)]
pub struct RuntimeSigner {
    backend: Arc<dyn SignerBackend>,
}

impl RuntimeSigner {
    #[must_use]
    pub fn new(backend: Arc<dyn SignerBackend>) -> Self {
        Self { backend }
    }

    pub(crate) fn from_starknet_signer<S>(signer: S, kind: SignerKind) -> Self
    where
        S: Signer + Send + Sync + 'static,
    {
        Self::new(Arc::new(StarknetSignerBackend::new(signer, kind)))
    }

    #[must_use]
    pub fn kind(&self) -> SignerKind {
        self.backend.kind()
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
        self.backend.public_key().await
    }

    async fn sign_hash(&self, hash: &Felt) -> Result<Signature, Self::SignError> {
        self.backend.sign_hash(hash).await
    }

    fn is_interactive(&self, context: SignerInteractivityContext<'_>) -> bool {
        self.backend.is_interactive(context)
    }
}

#[cfg(test)]
mod tests {
    use starknet_rust::signers::{LocalWallet, SigningKey};

    use super::*;

    #[tokio::test]
    async fn adapts_starknet_signers_to_one_error_type() {
        let key = SigningKey::from_secret_scalar(Felt::ONE);
        let expected_public_key = key.verifying_key();
        let signer = RuntimeSigner::from_starknet_signer(
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
