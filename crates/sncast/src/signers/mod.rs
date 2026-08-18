//! Persistent signer specifications and runtime signer implementations.

pub mod credentials;
pub mod derivation_path;
pub mod error;
pub mod keystore;
pub mod resolver;
pub mod runtime;
pub mod spec;

pub use credentials::{
    LEGACY_KEYSTORE_PASSWORD_ENV, SNCAST_KEYSTORE_PASSWORD_ENV, keystore_password,
};
pub use derivation_path::{DerivationPathError, parse_derivation_path, validate_derivation_path};
pub use error::SignerError;
pub use keystore::KeystoreFile;
pub use resolver::{
    KeystoreSignerProvider, SignerProvider, SignerProviderContext, SignerResolver,
    resolve_keystore_path,
};
pub use runtime::RuntimeSigner;
pub use spec::{KeystoreSpec, LedgerSpec, PrivateKeySpec, SignerKind, SignerSpec};
