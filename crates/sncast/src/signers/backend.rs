use async_trait::async_trait;
use starknet_rust::core::crypto::Signature;
use starknet_rust::signers::{Signer, SignerInteractivityContext, VerifyingKey};
use starknet_types_core::felt::Felt;

use crate::signers::{SignerError, SignerKind};

#[async_trait]
pub trait SignerBackend: Send + Sync {
    async fn public_key(&self) -> Result<VerifyingKey, SignerError>;
    async fn sign_hash(&self, hash: &Felt) -> Result<Signature, SignerError>;
    fn is_interactive(&self, context: SignerInteractivityContext<'_>) -> bool;
    fn kind(&self) -> SignerKind;
}

pub(crate) struct StarknetSignerBackend<S> {
    signer: S,
    kind: SignerKind,
}

impl<S> StarknetSignerBackend<S> {
    pub(crate) fn new(signer: S, kind: SignerKind) -> Self {
        Self { signer, kind }
    }
}

#[async_trait]
impl<S> SignerBackend for StarknetSignerBackend<S>
where
    S: Signer + Send + Sync,
{
    async fn public_key(&self) -> Result<VerifyingKey, SignerError> {
        self.signer
            .get_public_key()
            .await
            .map_err(|error| SignerError::Backend {
                kind: self.kind,
                operation: "get public key",
                message: error.to_string(),
            })
    }

    async fn sign_hash(&self, hash: &Felt) -> Result<Signature, SignerError> {
        self.signer
            .sign_hash(hash)
            .await
            .map_err(|error| SignerError::Backend {
                kind: self.kind,
                operation: "sign hash",
                message: error.to_string(),
            })
    }

    fn is_interactive(&self, context: SignerInteractivityContext<'_>) -> bool {
        self.signer.is_interactive(context)
    }

    fn kind(&self) -> SignerKind {
        self.kind
    }
}
