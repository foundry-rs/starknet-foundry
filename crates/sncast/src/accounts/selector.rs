use std::num::NonZeroU8;

use crate::accounts::{AccountName, AccountsError};

#[derive(Clone, Debug)]
pub enum AccountSelector {
    Named { name: AccountName },
    Devnet { index: NonZeroU8 },
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
            .and_then(NonZeroU8::new)
            .ok_or_else(|| AccountsError::InvalidAccount {
                message: format!("invalid devnet account selector `{value}`"),
            })?;
        Ok(Self::Devnet { index })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_nonzero_devnet_indices() {
        assert!(matches!(
            AccountSelector::devnet("devnet-1").unwrap(),
            AccountSelector::Devnet { .. }
        ));
        assert!(AccountSelector::devnet("devnet-0").is_err());
        assert!(AccountSelector::devnet("devnet-invalid").is_err());
    }
}
