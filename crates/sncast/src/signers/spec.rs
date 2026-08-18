use camino::Utf8PathBuf;
use starknet_rust::signers::DerivationPath;
use starknet_types_core::felt::Felt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignerKind {
    PrivateKey,
    Keystore,
    Ledger,
}

impl Display for SignerKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PrivateKey => formatter.write_str("private_key"),
            Self::Keystore => formatter.write_str("keystore"),
            Self::Ledger => formatter.write_str("ledger"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SignerSpec {
    PrivateKey(PrivateKeySpec),
    Keystore(KeystoreSpec),
    Ledger(LedgerSpec),
}

impl SignerSpec {
    #[must_use]
    pub fn kind(&self) -> SignerKind {
        match self {
            Self::PrivateKey(_) => SignerKind::PrivateKey,
            Self::Keystore(_) => SignerKind::Keystore,
            Self::Ledger(_) => SignerKind::Ledger,
        }
    }

    #[must_use]
    pub fn private_key(&self) -> Option<Felt> {
        match self {
            Self::PrivateKey(spec) => Some(spec.private_key()),
            Self::Keystore(_) | Self::Ledger(_) => None,
        }
    }

    #[must_use]
    pub fn derivation_path(&self) -> Option<&DerivationPath> {
        match self {
            Self::Ledger(spec) => Some(spec.derivation_path()),
            Self::PrivateKey(_) | Self::Keystore(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PrivateKeySpec {
    private_key: Felt,
}

impl PrivateKeySpec {
    #[must_use]
    pub fn new(private_key: Felt) -> Self {
        Self { private_key }
    }

    #[must_use]
    pub fn private_key(self) -> Felt {
        self.private_key
    }
}

#[derive(Clone, Debug)]
pub struct KeystoreSpec {
    path: Utf8PathBuf,
    password_env: Option<String>,
}

impl KeystoreSpec {
    #[must_use]
    pub fn new(path: Utf8PathBuf, password_env: Option<String>) -> Self {
        Self { path, password_env }
    }

    #[must_use]
    pub fn path(&self) -> &Utf8PathBuf {
        &self.path
    }

    #[must_use]
    pub fn password_env(&self) -> Option<&str> {
        self.password_env.as_deref()
    }
}

#[derive(Clone, Debug)]
pub struct LedgerSpec {
    derivation_path: DerivationPath,
}

impl LedgerSpec {
    #[must_use]
    pub fn new(derivation_path: DerivationPath) -> Self {
        Self { derivation_path }
    }

    #[must_use]
    pub fn derivation_path(&self) -> &DerivationPath {
        &self.derivation_path
    }
}
