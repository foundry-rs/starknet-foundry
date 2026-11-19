use std::env;
use std::io::{self, IsTerminal};

use crate::signers::KeystoreSpec;
use crate::signers::error::KeystoreError;

pub const SNCAST_KEYSTORE_PASSWORD_ENV: &str = "SNCAST_KEYSTORE_PASSWORD";
pub const LEGACY_KEYSTORE_PASSWORD_ENV: &str = "KEYSTORE_PASSWORD";
pub const LEGACY_CREATE_KEYSTORE_PASSWORD_ENV: &str = "CREATE_KEYSTORE_PASSWORD";

pub fn keystore_password(spec: &KeystoreSpec) -> Result<String, KeystoreError> {
    let from_env = spec
        .password_env()
        .into_iter()
        .chain([SNCAST_KEYSTORE_PASSWORD_ENV, LEGACY_KEYSTORE_PASSWORD_ENV])
        .find_map(|variable| env::var(variable).ok());

    if let Some(password) = from_env {
        return Ok(password);
    }

    if io::stdin().is_terminal() {
        return rpassword::prompt_password("Enter keystore password: ")
            .map_err(|source| KeystoreError::InteractivePassword { source });
    }

    Err(KeystoreError::PasswordUnavailable)
}
