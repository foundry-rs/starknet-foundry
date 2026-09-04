use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::accounts::{AccountRegistry, AccountsError, AccountsFileError};

pub mod conversion;
pub mod v1;
pub mod v2;

#[derive(Clone, Copy, Debug, strum_macros::Display, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceVersion {
    #[strum(to_string = "v1")]
    V1,

    #[strum(to_string = "v1")]
    V2,
}

impl SourceVersion {
    pub const LATEST: Self = Self::V2;

    pub fn is_latest(&self) -> bool {
        self == &Self::LATEST
    }
}

#[derive(Debug)]
enum VersionedAccountsFile {
    V1(v1::AccountsFile),
    V2(v2::AccountsFile),
}

impl VersionedAccountsFile {
    fn parse_json(input: &[u8]) -> Result<Self, AccountsFileError> {
        if input.iter().all(u8::is_ascii_whitespace) {
            // Empty files are valid legacy inputs and predate the versioned envelope.
            return Ok(Self::V1(v1::AccountsFile::default()));
        }

        let envelope: Value = deserialize(input)?;
        match envelope.get("version") {
            None | Some(Value::Object(_)) => deserialize(input).map(Self::V1),
            Some(version) if version.as_u64() == Some(u64::from(v2::VERSION)) => {
                let file_content: v2::AccountsFile = deserialize(input)?;
                Ok(Self::V2(file_content))
            }
            Some(version) => Err(AccountsFileError::Version {
                version: version.to_string(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct DecodedAccountRegistry {
    pub registry: AccountRegistry,
    pub source_version: SourceVersion,
}

impl DecodedAccountRegistry {
    pub fn parse_json(input: &[u8]) -> Result<Self, AccountsError> {
        match VersionedAccountsFile::parse_json(input)? {
            VersionedAccountsFile::V1(file_content) => Ok(Self {
                registry: file_content.try_into()?,
                source_version: SourceVersion::V1,
            }),
            VersionedAccountsFile::V2(file) => Ok(Self {
                registry: file.try_into()?,
                source_version: SourceVersion::V2,
            }),
        }
    }
}

fn deserialize<T: DeserializeOwned>(input: &[u8]) -> Result<T, AccountsFileError> {
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    let value = serde_path_to_error::deserialize(&mut deserializer).map_err(|source| {
        AccountsFileError::Schema {
            field_path: source.path().to_string(),
            source: source.into_inner(),
        }
    })?;
    deserializer
        .end()
        .map_err(|source| AccountsFileError::Deserialize { source })?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use camino::Utf8PathBuf;
    use serde_json::json;
    use starknet_types_core::felt::Felt;

    use super::*;
    use crate::accounts::{AccountName, AccountRecord, AccountType, NetworkName};
    use crate::signers::{KeystoreSpec, SignerSpec};

    #[test]
    fn decodes_existing_accounts_fixture_as_v1() {
        let decoded = DecodedAccountRegistry::parse_json(include_bytes!(
            "../../../tests/data/accounts/accounts.json"
        ))
        .unwrap();

        assert_eq!(decoded.source_version, SourceVersion::V1);
        assert!(decoded.registry.account("alpha-sepolia", "user1").is_some());
    }

    #[test]
    fn decodes_empty_file_as_empty_v1() {
        let decoded = DecodedAccountRegistry::parse_json(b" \n\t").unwrap();

        assert_eq!(decoded.source_version, SourceVersion::V1);
        assert!(decoded.registry.networks().is_empty());
    }

    #[test]
    fn decodes_object_valued_version_key_as_v1_network() {
        let input = br#"{
            "version": {
                "alice": {
                    "public_key": "0x1",
                    "address": "0x3",
                    "private_key": "0x2"
                }
            }
        }"#;

        let decoded = DecodedAccountRegistry::parse_json(input).unwrap();

        assert_eq!(decoded.source_version, SourceVersion::V1);
        assert!(decoded.registry.account("version", "alice").is_some());
    }

    #[test]
    fn preserves_v1_untagged_precedence_only_in_v1() {
        let input = br#"{
            "alpha-sepolia": {
                "alice": {
                    "public_key": "0x1",
                    "address": "0x3",
                    "private_key": "0x2",
                    "ledger_path": "m/44'/60'/0'/0/0"
                }
            }
        }"#;

        let decoded = DecodedAccountRegistry::parse_json(input).unwrap();
        assert!(matches!(
            &decoded
                .registry
                .account("alpha-sepolia", "alice")
                .unwrap()
                .signer,
            SignerSpec::PrivateKey(_)
        ));
    }

    #[test]
    fn decodes_tagged_v2_keystore() {
        let input = br#"{
            "version": 2,
            "accounts": {
                "alpha-sepolia": {
                    "alice": {
                        "public_key": "0x1",
                        "address": "0x3",
                        "signer": {
                            "type": "keystore",
                            "path": "keys/alice.json",
                            "password_env": "ALICE_PASSWORD"
                        }
                    }
                }
            }
        }"#;

        let decoded = DecodedAccountRegistry::parse_json(input).unwrap();
        assert_eq!(decoded.source_version, SourceVersion::V2);
        assert!(matches!(
            &decoded
                .registry
                .account("alpha-sepolia", "alice")
                .unwrap()
                .signer,
            SignerSpec::Keystore(_)
        ));
    }

    #[test]
    fn decodes_tagged_v2_private_key() {
        let input = br#"{
            "version": 2,
            "accounts": {
                "alpha-sepolia": {
                    "alice": {
                        "public_key": "0x1",
                        "address": "0x3",
                        "signer": {
                            "type": "private_key",
                            "private_key": "0x2"
                        }
                    }
                }
            }
        }"#;

        let decoded = DecodedAccountRegistry::parse_json(input).unwrap();

        assert_eq!(decoded.source_version, SourceVersion::V2);
        assert!(matches!(
            &decoded
                .registry
                .account("alpha-sepolia", "alice")
                .unwrap()
                .signer,
            SignerSpec::PrivateKey(_)
        ));
    }

    #[test]
    fn decodes_tagged_v2_ledger() {
        let input = br#"{
            "version": 2,
            "accounts": {
                "alpha-sepolia": {
                    "alice": {
                        "public_key": "0x1",
                        "address": "0x3",
                        "signer": {
                            "type": "ledger",
                            "derivation_path": "m/2645'/1195502025'/355113700'/0'/0'/0"
                        }
                    }
                }
            }
        }"#;

        let decoded = DecodedAccountRegistry::parse_json(input).unwrap();

        assert_eq!(decoded.source_version, SourceVersion::V2);
        assert!(matches!(
            &decoded
                .registry
                .account("alpha-sepolia", "alice")
                .unwrap()
                .signer,
            SignerSpec::Ledger(_)
        ));
    }

    #[test]
    fn rejects_unknown_version() {
        let unknown_version = serde_json::to_vec(&json!({"version": 3, "accounts": {}})).unwrap();
        assert!(matches!(
            DecodedAccountRegistry::parse_json(&unknown_version),
            Err(AccountsError::AccountsFile(
                AccountsFileError::Version { .. }
            ))
        ));
    }

    #[test]
    fn rejects_untagged_signer() {
        let untagged = br#"{
            "version": 2,
            "accounts": {
                "alpha-sepolia": {
                    "alice": {
                        "public_key": "0x1",
                        "address": "0x3",
                        "private_key": "0x2"
                    }
                }
            }
        }"#;
        assert!(matches!(
            DecodedAccountRegistry::parse_json(untagged),
            Err(AccountsError::AccountsFile(
                AccountsFileError::Schema { .. }
            ))
        ));
    }

    #[test]
    fn rejects_trailing_json_content() {
        let input = br#"{"version": 2, "accounts": {}} {"trailing": true}"#;

        assert!(matches!(
            DecodedAccountRegistry::parse_json(input),
            Err(AccountsError::AccountsFile(
                AccountsFileError::Deserialize { .. }
            ))
        ));
    }

    #[test]
    fn rejects_missing_address_in_both_schemas() {
        let v1 = br#"{
            "alpha-sepolia": {
                "alice": {"public_key": "0x1", "private_key": "0x2"}
            }
        }"#;
        let v2 = br#"{
            "version": 2,
            "accounts": {
                "alpha-sepolia": {
                    "alice": {
                        "public_key": "0x1",
                        "signer": {"type": "private_key", "private_key": "0x2"}
                    }
                }
            }
        }"#;

        for input in [v1.as_slice(), v2.as_slice()] {
            assert!(matches!(
                DecodedAccountRegistry::parse_json(input),
                Err(AccountsError::AccountsFile(
                    AccountsFileError::Schema { .. }
                ))
            ));
        }
    }

    #[test]
    fn encodes_only_deterministic_v2() {
        let account = AccountRecord {
            public_key: Felt::ONE,
            address: Felt::TWO,
            salt: None,
            deployed: None,
            class_hash: None,
            legacy: Some(false),
            account_type: Some(AccountType::OpenZeppelin),
            signer: SignerSpec::Keystore(KeystoreSpec::new(
                Utf8PathBuf::from("keys/alice.json"),
                Some("ALICE_PASSWORD".to_owned()),
            )),
        };
        let registry = AccountRegistry::new(BTreeMap::from([(
            NetworkName::new("alpha-sepolia").unwrap(),
            BTreeMap::from([(AccountName::new("alice").unwrap(), account)]),
        )]));

        let encoded = registry.encode_v2().unwrap();
        let value: Value = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(value["version"], 2);
        assert_eq!(
            value["accounts"]["alpha-sepolia"]["alice"]["signer"]["type"],
            "keystore"
        );
        assert_eq!(encoded, registry.encode_v2().unwrap());
    }
}
