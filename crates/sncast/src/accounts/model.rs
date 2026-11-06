use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;

use conversions::felt::ToShortString;

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

/// Chain ID of a Starknet network, written as a Cairo short string (e.g. `SN_SEPOLIA`) whenever it
/// is printable and as `0x`-prefixed hex otherwise.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ChainId(Felt);

impl ChainId {
    /// Maps network names used as keys by legacy accounts files to their chain IDs.
    #[must_use]
    pub fn from_legacy_network_name(name: &str) -> Option<Self> {
        let chain_id = match name {
            "alpha-mainnet" => "SN_MAIN",
            "alpha-sepolia" => "SN_SEPOLIA",
            "alpha-integration-sepolia" => "SN_INTEGRATION_SEPOLIA",
            _ => return None,
        };
        Some(Self(Felt::from_bytes_be_slice(chain_id.as_bytes())))
    }
}

impl From<Felt> for ChainId {
    fn from(value: Felt) -> Self {
        Self(value)
    }
}

impl FromStr for ChainId {
    type Err = AccountsError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let chain_id = if value.starts_with("0x") {
            Felt::from_hex(value).ok()
        } else if !value.is_empty() && value.len() <= 31 && value.is_ascii() {
            Some(Felt::from_bytes_be_slice(value.as_bytes()))
        } else {
            None
        };

        chain_id
            .map(Self)
            .ok_or_else(|| AccountsError::InvalidChainId {
                value: value.to_owned(),
            })
    }
}

impl Display for ChainId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0.to_short_string() {
            // Short strings starting with `0x` would be parsed back as hex.
            Ok(short_string)
                if !short_string.is_empty()
                    && !short_string.starts_with("0x")
                    && short_string
                        .bytes()
                        .all(|byte| byte == b' ' || byte.is_ascii_graphic()) =>
            {
                formatter.write_str(&short_string)
            }
            _ => write!(formatter, "{:#x}", self.0),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AccountRegistry {
    networks: BTreeMap<ChainId, BTreeMap<AccountName, AccountRecord>>,
}

impl AccountRegistry {
    #[must_use]
    pub fn new(networks: BTreeMap<ChainId, BTreeMap<AccountName, AccountRecord>>) -> Self {
        Self { networks }
    }

    #[must_use]
    pub fn networks(&self) -> &BTreeMap<ChainId, BTreeMap<AccountName, AccountRecord>> {
        &self.networks
    }

    pub fn networks_mut(&mut self) -> &mut BTreeMap<ChainId, BTreeMap<AccountName, AccountRecord>> {
        &mut self.networks
    }

    #[must_use]
    pub fn account(&self, chain_id: &ChainId, name: &AccountName) -> Option<&AccountRecord> {
        self.networks
            .get(chain_id)
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
        assert!(AccountName::new("  \t").is_err());
    }

    #[test]
    fn chain_ids_round_trip_through_text() {
        let sepolia = Felt::from_bytes_be_slice(b"SN_SEPOLIA");
        let cases = [
            (sepolia, "SN_SEPOLIA"),
            (Felt::from_bytes_be_slice(b"my chain"), "my chain"),
            // Not printable ASCII.
            (Felt::from_bytes_be_slice(b"SN\0X"), "0x534e0058"),
            (Felt::from_bytes_be_slice(b"\x01\x02"), "0x102"),
            (Felt::ZERO, "0x0"),
            // Would read back as hex.
            (Felt::from_bytes_be_slice(b"0x1"), "0x307831"),
        ];

        for (felt, text) in cases {
            let chain_id = ChainId::from(felt);
            assert_eq!(chain_id.to_string(), text);
            assert_eq!(text.parse::<ChainId>().unwrap(), chain_id);
        }
        assert_eq!(
            "0x534e5f5345504f4c4941".parse::<ChainId>().unwrap(),
            ChainId::from(sepolia)
        );
    }

    #[test]
    fn chain_ids_reject_invalid_text() {
        for text in ["", "0xnot-hex", "a".repeat(32).as_str(), "sepolia_ł"] {
            assert!(matches!(
                text.parse::<ChainId>(),
                Err(AccountsError::InvalidChainId { value }) if value == text
            ));
        }
    }

    #[test]
    fn maps_legacy_network_names_to_chain_ids() {
        for (name, chain_id) in [
            ("alpha-mainnet", "SN_MAIN"),
            ("alpha-sepolia", "SN_SEPOLIA"),
            ("alpha-integration-sepolia", "SN_INTEGRATION_SEPOLIA"),
        ] {
            assert_eq!(
                ChainId::from_legacy_network_name(name).unwrap().to_string(),
                chain_id
            );
        }
        assert_eq!(ChainId::from_legacy_network_name("SN_SEPOLIA"), None);
    }

    #[test]
    fn registry_looks_up_accounts_by_identifiers() {
        let network = "SN_SEPOLIA".parse::<ChainId>().unwrap();
        let mut networks = BTreeMap::new();
        networks.insert(
            network,
            BTreeMap::from([(AccountName::new("alice").unwrap(), account())]),
        );

        let registry = AccountRegistry::new(networks);
        assert!(
            registry
                .account(&network, &AccountName::new("alice").unwrap())
                .is_some()
        );
        assert!(
            registry
                .account(&network, &AccountName::new("bob").unwrap())
                .is_none()
        );
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
