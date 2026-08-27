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

    #[error("failed to create keystore at {path}: {source}")]
    Create {
        path: Utf8PathBuf,
        #[source]
        source: StarknetKeystoreError,
    },

    #[error(
        "failed to create the keystore file's parent directory at {keystore_file_parent_path}: {source}"
    )]
    CreateDirectory {
        keystore_file_parent_path: Utf8PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to remove the keystore from {path}: {source}")]
    Remove {
        path: Utf8PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to inspect the keystore at {path}: {source}")]
    Inspect {
        path: Utf8PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("failed to set secret permissions for the keystore at {path}: {source}")]
    SetSecretPermissions {
        path: Utf8PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("keystore at {path} already exists")]
    AlreadyExists { path: Utf8PathBuf },
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
