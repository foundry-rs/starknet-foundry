//! Persistent signer specifications and runtime signer implementations.

pub mod credentials;
pub mod error;
pub mod runtime;
pub mod spec;

pub use credentials::{
    LEGACY_KEYSTORE_PASSWORD_ENV, SNCAST_KEYSTORE_PASSWORD_ENV, keystore_password,
};
pub use error::{KeystoreError, SignerError, SignerOperationError};
pub use runtime::RuntimeSigner;
pub use spec::{KeystoreSpec, LedgerSpec, PrivateKeySpec, SignerKind, SignerSpec};
