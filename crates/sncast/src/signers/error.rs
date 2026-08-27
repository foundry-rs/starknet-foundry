use std::io;

use crate::{helpers::ledger::LedgerError, signers::SignerKind};
use camino::Utf8PathBuf;
use starknet_rust::signers::KeystoreError as StarknetKeystoreError;
use starknet_types_core::felt::Felt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignerError {
    #[error(transparent)]
    Keystore(#[from] KeystoreError),

    #[error(transparent)]
    Ledger(#[from] LedgerError),

    #[error(transparent)]
    Operation(#[from] SignerOperationError),

    #[error(
        "{kind} signer public key does not match the account: expected {expected:#x}, got {actual:#x}"
    )]
    PublicKeyMismatch {
        kind: SignerKind,
        expected: Felt,
        actual: Felt,
    },
}

#[derive(Debug, Error)]
pub enum KeystoreError {
    #[error(
        "no keystore password is available; configure the signer's `password_env`, set SNCAST_KEYSTORE_PASSWORD, or use an interactive terminal"
    )]
    PasswordUnavailable,

    #[error("failed to read keystore password from the terminal")]
    InteractivePassword {
        #[source]
        source: io::Error,
    },

    #[error("failed to decrypt keystore `{path}`")]
    Decrypt {
        path: Utf8PathBuf,
        #[source]
        source: StarknetKeystoreError,
    },
}

#[derive(Debug, Error)]
pub enum SignerOperationError {
    #[error("{kind} signer failed to obtain its public key")]
    GetPublicKey {
        kind: SignerKind,
        #[source]
        source: anyhow::Error,
    },
    #[error("{kind} signer failed to sign hash")]
    SignHash {
        kind: SignerKind,
        #[source]
        source: anyhow::Error,
    },
}
