use std::env;
use std::io::{self, IsTerminal};

use crate::signers::KeystoreSpec;
use crate::signers::error::KeystoreError;

pub const SNCAST_KEYSTORE_PASSWORD_ENV: &str = "SNCAST_KEYSTORE_PASSWORD";

pub fn keystore_password(spec: &KeystoreSpec) -> Result<String, KeystoreError> {
    read_keystore_password(spec, false)
}

/// Confirms terminal-entered passwords for a new keystore; environment values need no confirmation.
pub fn new_keystore_password(spec: &KeystoreSpec) -> Result<String, KeystoreError> {
    read_keystore_password(spec, true)
}

fn read_keystore_password(spec: &KeystoreSpec, confirm: bool) -> Result<String, KeystoreError> {
    let from_env = spec
        .password_env()
        .into_iter()
        .chain([SNCAST_KEYSTORE_PASSWORD_ENV])
        .find_map(|variable| env::var(variable).ok());

    if let Some(password) = from_env {
        return Ok(password);
    }

    if io::stdin().is_terminal() {
        let password = rpassword::prompt_password("Enter keystore password: ")
            .map_err(|source| KeystoreError::InteractivePassword { source })?;
        if confirm {
            let confirmation = rpassword::prompt_password("Confirm keystore password: ")
                .map_err(|source| KeystoreError::InteractivePassword { source })?;
            if password != confirmation {
                return Err(KeystoreError::PasswordMismatch);
            }
        }
        return Ok(password);
    }

    Err(KeystoreError::PasswordUnavailable)
}
