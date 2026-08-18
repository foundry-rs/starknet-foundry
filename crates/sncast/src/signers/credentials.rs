use std::env;
use std::io::{self, IsTerminal};

use crate::signers::{KeystoreSpec, SignerError};

pub const SNCAST_KEYSTORE_PASSWORD_ENV: &str = "SNCAST_KEYSTORE_PASSWORD";
pub const LEGACY_KEYSTORE_PASSWORD_ENV: &str = "KEYSTORE_PASSWORD";

pub trait CredentialProvider: Send + Sync {
    fn keystore_password(&self, spec: &KeystoreSpec) -> Result<String, SignerError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultCredentialProvider;

impl CredentialProvider for DefaultCredentialProvider {
    fn keystore_password(&self, spec: &KeystoreSpec) -> Result<String, SignerError> {
        let variables = spec
            .password_env()
            .into_iter()
            .chain([SNCAST_KEYSTORE_PASSWORD_ENV, LEGACY_KEYSTORE_PASSWORD_ENV]);

        for variable in variables {
            if let Ok(password) = env::var(variable) {
                return Ok(password);
            }
        }

        if io::stdin().is_terminal() {
            return rpassword::prompt_password("Enter keystore password: ").map_err(|error| {
                SignerError::Backend {
                    kind: crate::signers::SignerKind::Keystore,
                    operation: "read password",
                    message: error.to_string(),
                }
            });
        }

        Err(SignerError::CredentialUnavailable)
    }
}
