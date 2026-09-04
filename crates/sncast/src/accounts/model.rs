use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;

use crate::accounts::{AccountsError, AccountsFileError, schema};
use crate::signers::SignerSpec;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    #[serde(rename = "open_zeppelin")]
    OpenZeppelin,
    // Backwards compatibility with pre-rebranding account files.
    #[serde(alias = "argent")]
    Ready,
    Braavos,
}

impl FromStr for AccountType {
    type Err = AccountsError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "open_zeppelin" | "open-zeppelin" | "oz" => Ok(Self::OpenZeppelin),
            "ready" => Ok(Self::Ready),
            "braavos" => Ok(Self::Braavos),
            account_type => Err(AccountsError::InvalidAccountType {
                account_type: account_type.to_owned(),
            }),
        }
    }
}

impl Display for AccountType {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

macro_rules! identifier {
    ($name:ident, $kind:literal) => {
        #[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, AccountsError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(AccountsError::EmptyIdentifier { kind: $kind });
                }
                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Borrow<str> for $name {
            fn borrow(&self) -> &str {
                self.as_str()
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl TryFrom<String> for $name {
            type Error = AccountsError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = AccountsError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

identifier!(AccountName, "account name");
identifier!(NetworkName, "network name");

#[derive(Clone, Debug, Default)]
pub struct AccountRegistry {
    networks: BTreeMap<NetworkName, BTreeMap<AccountName, AccountRecord>>,
}

impl AccountRegistry {
    #[must_use]
    pub fn new(networks: BTreeMap<NetworkName, BTreeMap<AccountName, AccountRecord>>) -> Self {
        Self { networks }
    }

    #[must_use]
    pub fn networks(&self) -> &BTreeMap<NetworkName, BTreeMap<AccountName, AccountRecord>> {
        &self.networks
    }

    pub fn networks_mut(
        &mut self,
    ) -> &mut BTreeMap<NetworkName, BTreeMap<AccountName, AccountRecord>> {
        &mut self.networks
    }

    #[must_use]
    pub fn account(&self, network: &str, name: &str) -> Option<&AccountRecord> {
        self.networks
            .get(network)
            .and_then(|accounts| accounts.get(name))
    }

    pub fn encode(&self) -> Result<Vec<u8>, AccountsError> {
        let file_content = schema::v2::AccountsFile::from(self);
        let mut output = serde_json::to_vec_pretty(&file_content)
            .map_err(|source| AccountsFileError::Serialize { source })?;
        output.push(b'\n');
        Ok(output)
    }
}

#[derive(Clone, Debug)]
pub struct AccountRecord {
    pub public_key: Felt,
    pub address: Felt,
    pub salt: Option<Felt>,
    pub deployed: Option<bool>,
    pub class_hash: Option<Felt>,
    pub legacy: Option<bool>,
    pub account_type: Option<AccountType>,
    pub signer: SignerSpec,
}

impl AccountRecord {
    pub fn as_deployable(&self) -> Result<DeployableAccountRecord<'_>, AccountsError> {
        Ok(DeployableAccountRecord {
            account: self,
            salt: required(self.salt, "salt", "account deployment")?,
            class_hash: required(self.class_hash, "class_hash", "account deployment")?,
            account_type: required(self.account_type, "type", "account deployment")?,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DeployableAccountRecord<'a> {
    account: &'a AccountRecord,
    salt: Felt,
    class_hash: Felt,
    account_type: AccountType,
}

impl<'a> DeployableAccountRecord<'a> {
    #[must_use]
    pub fn account(self) -> &'a AccountRecord {
        self.account
    }

    #[must_use]
    pub fn salt(self) -> Felt {
        self.salt
    }

    #[must_use]
    pub fn class_hash(self) -> Felt {
        self.class_hash
    }

    #[must_use]
    pub fn account_type(self) -> AccountType {
        self.account_type
    }
}

fn required<T>(
    value: Option<T>,
    field: &'static str,
    operation: &'static str,
) -> Result<T, AccountsError> {
    value.ok_or(AccountsError::MissingField { field, operation })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signers::PrivateKeySpec;

    fn account() -> AccountRecord {
        AccountRecord {
            public_key: Felt::ONE,
            address: Felt::TWO,
            salt: Some(Felt::THREE),
            deployed: Some(false),
            class_hash: Some(Felt::from(4_u8)),
            legacy: Some(false),
            account_type: Some(AccountType::OpenZeppelin),
            signer: SignerSpec::PrivateKey(PrivateKeySpec::new(Felt::from(5_u8))),
        }
    }

    #[test]
    fn identifiers_reject_empty_values() {
        assert!(AccountName::new("").is_err());
        assert!(NetworkName::new("").is_err());
        assert!(AccountName::new("  \t").is_err());
        assert!(NetworkName::new("\n").is_err());
    }

    #[test]
    fn registry_looks_up_accounts_by_strings() {
        let mut networks = BTreeMap::new();
        networks.insert(
            NetworkName::new("alpha-sepolia").unwrap(),
            BTreeMap::from([(AccountName::new("alice").unwrap(), account())]),
        );

        let registry = AccountRegistry::new(networks);
        assert!(registry.account("alpha-sepolia", "alice").is_some());
        assert!(registry.account("alpha-sepolia", "bob").is_none());
    }

    #[test]
    fn deployable_view_validates_required_fields() {
        let account = account();
        assert_eq!(account.as_deployable().unwrap().salt(), Felt::THREE);

        let incomplete = AccountRecord {
            salt: None,
            ..account
        };
        assert!(matches!(
            incomplete.as_deployable(),
            Err(AccountsError::MissingField { field: "salt", .. })
        ));
    }
}
