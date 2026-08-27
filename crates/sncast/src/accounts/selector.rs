use camino::Utf8PathBuf;

use crate::accounts::{AccountName, AccountsError};

#[derive(Clone, Debug)]
pub enum AccountSelector {
    Named {
        name: AccountName,
    },
    Devnet {
        index: u8,
    },
    LegacyStarkli {
        account_file: Utf8PathBuf,
        keystore_file: Utf8PathBuf,
    },
}

impl AccountSelector {
    pub fn named(name: impl Into<String>) -> Result<Self, AccountsError> {
        Ok(Self::Named {
            name: AccountName::new(name)?,
        })
    }

    pub fn devnet(value: &str) -> Result<Self, AccountsError> {
        let index = value
            .strip_prefix("devnet-")
            .and_then(|value| value.parse::<u8>().ok())
            .ok_or_else(|| AccountsError::InvalidAccount {
                message: format!(
                    "expected `devnet-<index>` with an index fitting in u8; got `{value}`"
                ),
            })?;
        Ok(Self::Devnet { index })
    }

    #[must_use]
    pub fn legacy_starkli(account_file: Utf8PathBuf, keystore_file: Utf8PathBuf) -> Self {
        Self::LegacyStarkli {
            account_file,
            keystore_file,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_devnet_indices() {
        assert!(matches!(
            AccountSelector::devnet("devnet-1").unwrap(),
            AccountSelector::Devnet { .. }
        ));
        // Range validation is left to the devnet account lookup
        assert!(matches!(
            AccountSelector::devnet("devnet-0").unwrap(),
            AccountSelector::Devnet { index: 0 }
        ));
        assert!(AccountSelector::devnet("devnet-invalid").is_err());
    }
}
