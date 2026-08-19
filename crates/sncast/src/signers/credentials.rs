use std::env;
use std::io::{self, IsTerminal};

use crate::signers::{KeystoreSpec, SignerError};

pub const SNCAST_KEYSTORE_PASSWORD_ENV: &str = "SNCAST_KEYSTORE_PASSWORD";
pub const LEGACY_KEYSTORE_PASSWORD_ENV: &str = "KEYSTORE_PASSWORD";

pub fn keystore_password(spec: &KeystoreSpec) -> Result<String, SignerError> {
    if let Some(password) = password_from_environment(spec, |variable| env::var(variable).ok()) {
        return Ok(password);
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

fn password_from_environment(
    spec: &KeystoreSpec,
    mut lookup: impl FnMut(&str) -> Option<String>,
) -> Option<String> {
    spec.password_env()
        .into_iter()
        .chain([SNCAST_KEYSTORE_PASSWORD_ENV, LEGACY_KEYSTORE_PASSWORD_ENV])
        .find_map(|variable| lookup(variable))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use camino::Utf8PathBuf;

    use super::*;

    #[test]
    fn password_sources_have_stable_precedence() {
        let spec = KeystoreSpec::new(
            Utf8PathBuf::from("key.json"),
            Some("ACCOUNT_PASSWORD".to_owned()),
        );
        let values = HashMap::from([
            ("ACCOUNT_PASSWORD", "account-specific"),
            (SNCAST_KEYSTORE_PASSWORD_ENV, "sncast"),
            (LEGACY_KEYSTORE_PASSWORD_ENV, "legacy"),
        ]);

        let password =
            password_from_environment(&spec, |name| values.get(name).map(ToString::to_string));

        assert_eq!(password.as_deref(), Some("account-specific"));
    }

    #[test]
    fn falls_back_from_sncast_to_legacy_password_variable() {
        let spec = KeystoreSpec::new(Utf8PathBuf::from("key.json"), None);
        let values = HashMap::from([(LEGACY_KEYSTORE_PASSWORD_ENV, "legacy")]);

        let password =
            password_from_environment(&spec, |name| values.get(name).map(ToString::to_string));

        assert_eq!(password.as_deref(), Some("legacy"));
    }
}
