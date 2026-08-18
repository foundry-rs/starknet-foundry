use camino::Utf8PathBuf;
use starknet_types_core::felt::Felt;
use thiserror::Error;

use crate::signers::SignerKind;

#[derive(Debug, Error)]
pub enum SignerError {
    #[error("no runtime provider is registered for signer type `{kind}`")]
    Unsupported { kind: SignerKind },

    #[error(
        "no keystore password is available; configure the signer's `password_env`, set SNCAST_KEYSTORE_PASSWORD, or use an interactive terminal"
    )]
    CredentialUnavailable,

    #[error("failed to decrypt keystore `{path}`: {message}")]
    InvalidKeystore { path: Utf8PathBuf, message: String },

    #[error(
        "{kind} signer public key does not match the account: expected {expected:#x}, got {actual:#x}"
    )]
    PublicKeyMismatch {
        kind: SignerKind,
        expected: Felt,
        actual: Felt,
    },

    #[error("{kind} signer failed to {operation}: {message}")]
    Backend {
        kind: SignerKind,
        operation: &'static str,
        message: String,
    },
}
