use starknet_rust::core::crypto::Signature;
use starknet_rust::signers::{
    LedgerSigner, LocalWallet, Signer, SignerInteractivityContext, VerifyingKey,
};
use starknet_types_core::felt::Felt;

use crate::helpers::ledger::SncastLedgerTransport;
use crate::signers::{SignerError, SignerKind};

pub(crate) enum SignerBackend {
    LocalWallet {
        signer: LocalWallet,
        kind: SignerKind,
    },
    Ledger {
        signer: LedgerSigner<SncastLedgerTransport>,
        kind: SignerKind,
    },
}

impl SignerBackend {
    pub(crate) fn local_wallet(signer: LocalWallet, kind: SignerKind) -> Self {
        Self::LocalWallet { signer, kind }
    }

    pub(crate) fn ledger(signer: LedgerSigner<SncastLedgerTransport>, kind: SignerKind) -> Self {
        Self::Ledger { signer, kind }
    }

    pub(crate) async fn public_key(&self) -> Result<VerifyingKey, SignerError> {
        match self {
            Self::LocalWallet { signer, kind } => get_public_key(signer, *kind).await,
            Self::Ledger { signer, kind } => get_public_key(signer, *kind).await,
        }
    }

    pub(crate) async fn sign_hash(&self, hash: &Felt) -> Result<Signature, SignerError> {
        match self {
            Self::LocalWallet { signer, kind } => sign_hash(signer, hash, *kind).await,
            Self::Ledger { signer, kind } => sign_hash(signer, hash, *kind).await,
        }
    }

    pub(crate) fn is_interactive(&self, context: SignerInteractivityContext<'_>) -> bool {
        match self {
            Self::LocalWallet { signer, .. } => signer.is_interactive(context),
            Self::Ledger { signer, .. } => signer.is_interactive(context),
        }
    }

    pub(crate) fn kind(&self) -> SignerKind {
        match self {
            Self::LocalWallet { kind, .. } | Self::Ledger { kind, .. } => *kind,
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
