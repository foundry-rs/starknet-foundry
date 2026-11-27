use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use serde_json::error::Category;

use crate::accounts::{AccountRegistry, AccountsFileError};

pub mod conversion;
pub mod v1;
pub mod v2;

#[derive(Clone, Copy, Debug, strum_macros::Display, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceVersion {
    #[strum(to_string = "v1")]
    V1,

    #[strum(to_string = "v2")]
    V2,
}

impl SourceVersion {
    pub const LATEST: Self = Self::V2;

    #[must_use]
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
    pub fn parse_json(input: &[u8]) -> Result<Self, AccountsFileError> {
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
    let value = serde_path_to_error::deserialize(&mut deserializer).map_err(|error| {
        // The root path is printed as `.`, which tells the user nothing.
        let field_path = error
            .path()
            .iter()
            .next()
            .is_some()
            .then(|| error.path().to_string());
        classify(error.into_inner(), field_path)
    })?;
    deserializer.end().map_err(|error| classify(error, None))?;
    Ok(value)
}

fn classify(error: serde_json::Error, field_path: Option<String>) -> AccountsFileError {
    match error.classify() {
        Category::Data => AccountsFileError::Schema { field_path, error },
        Category::Syntax | Category::Eof | Category::Io => AccountsFileError::InvalidJson { error },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use camino::Utf8PathBuf;
    use serde_json::json;
    use starknet_types_core::felt::Felt;

    use super::*;
    use crate::accounts::{AccountName, AccountRecord, AccountType, ChainId};
    use crate::signers::{KeystoreSpec, SignerSpec};

    fn account<'a>(
        decoded: &'a DecodedAccountRegistry,
        chain_id: &str,
        name: &str,
    ) -> Option<&'a AccountRecord> {
        decoded.registry.account(
            &chain_id.parse::<ChainId>().unwrap(),
            &AccountName::new(name).unwrap(),
        )
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
        assert!(account(&decoded, "version", "alice").is_some());
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
            &account(&decoded, "SN_SEPOLIA", "alice").unwrap().signer,
            SignerSpec::PrivateKey(_)
        ));
    }

    #[test]
    fn decodes_tagged_v2_keystore() {
        let input = br#"{
            "version": 2,
            "accounts": {
                "SN_SEPOLIA": {
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
            &account(&decoded, "SN_SEPOLIA", "alice").unwrap().signer,
            SignerSpec::Keystore(_)
        ));
    }

    #[test]
    fn decodes_tagged_v2_private_key() {
        let input = br#"{
            "version": 2,
            "accounts": {
                "SN_SEPOLIA": {
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
            &account(&decoded, "SN_SEPOLIA", "alice").unwrap().signer,
            SignerSpec::PrivateKey(_)
        ));
    }

    #[test]
    fn decodes_tagged_v2_ledger() {
        let input = br#"{
            "version": 2,
            "accounts": {
                "SN_SEPOLIA": {
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
            &account(&decoded, "SN_SEPOLIA", "alice").unwrap().signer,
            SignerSpec::Ledger(_)
        ));
    }

    #[test]
    fn rejects_unknown_version() {
        let unknown_version = serde_json::to_vec(&json!({"version": 3, "accounts": {}})).unwrap();
        assert!(matches!(
            DecodedAccountRegistry::parse_json(&unknown_version),
            Err(AccountsFileError::Version { .. })
        ));
    }

    #[test]
    fn rejects_untagged_signer() {
        let untagged = br#"{
            "version": 2,
            "accounts": {
                "SN_SEPOLIA": {
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
            Err(AccountsFileError::Schema { .. })
        ));
    }

    #[test]
    fn rejects_trailing_json_content() {
        let input = br#"{"version": 2, "accounts": {}} {"trailing": true}"#;

        assert!(matches!(
            DecodedAccountRegistry::parse_json(input),
            Err(AccountsFileError::InvalidJson { .. })
        ));
    }

    #[test]
    fn distinguishes_invalid_json_from_invalid_values() {
        for input in [b"not valid JSON".as_slice(), b"{".as_slice()] {
            assert!(matches!(
                DecodedAccountRegistry::parse_json(input),
                Err(AccountsFileError::InvalidJson { .. })
            ));
        }

        let invalid_signer = br#"{
            "version": 2,
            "accounts": {
                "SN_SEPOLIA": {
                    "alice": {"public_key": "0x1", "address": "0x3", "signer": {}}
                }
            }
        }"#;
        for (input, expected_path) in [
            (
                invalid_signer.as_slice(),
                Some("accounts.SN_SEPOLIA.alice.signer"),
            ),
            (b"[]".as_slice(), None),
        ] {
            let Err(AccountsFileError::Schema { field_path, .. }) =
                DecodedAccountRegistry::parse_json(input)
            else {
                panic!("expected a schema error");
            };
            assert_eq!(field_path.as_deref(), expected_path);
        }
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
                "SN_SEPOLIA": {
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
                Err(AccountsFileError::Schema { .. })
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
            "SN_SEPOLIA".parse::<ChainId>().unwrap(),
            BTreeMap::from([(AccountName::new("alice").unwrap(), account)]),
        )]));

        let encoded = registry.encode().unwrap();
        let value: Value = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(value["version"], 2);
        assert_eq!(
            value["accounts"]["SN_SEPOLIA"]["alice"]["signer"]["type"],
            "keystore"
        );
        assert_eq!(encoded, registry.encode().unwrap());
    }

    #[test]
    fn converts_v1_network_names_to_chain_ids() {
        let input = br#"{
            "alpha-mainnet": {
                "alice": {"public_key": "0x1", "address": "0x3", "private_key": "0x2"}
            },
            "KATANA": {
                "bob": {"public_key": "0x1", "address": "0x3", "private_key": "0x2"}
            },
            "0x1234": {
                "carol": {"public_key": "0x1", "address": "0x3", "private_key": "0x2"}
            }
        }"#;

        let decoded = DecodedAccountRegistry::parse_json(input).unwrap();
        assert!(account(&decoded, "SN_MAIN", "alice").is_some());
        assert!(account(&decoded, "KATANA", "bob").is_some());
        assert!(account(&decoded, "0x1234", "carol").is_some());

        let encoded: Value = serde_json::from_slice(&decoded.registry.encode().unwrap()).unwrap();
        let mut keys: Vec<_> = encoded["accounts"].as_object().unwrap().keys().collect();
        keys.sort();
        assert_eq!(keys, ["0x1234", "KATANA", "SN_MAIN"]);
    }

    #[test]
    fn rejects_networks_that_are_not_chain_ids() {
        for network in ["a".repeat(32), "sepolia-ł".to_owned()] {
            let input = serde_json::to_vec(&json!({ network.clone(): {} })).unwrap();
            assert!(matches!(
                DecodedAccountRegistry::parse_json(&input),
                Err(AccountsFileError::InvalidNetworkKey { network: key }) if key == network
            ));
        }
    }

    #[test]
    fn rejects_networks_with_the_same_chain_id() {
        let v1 = json!({"alpha-sepolia": {}, "SN_SEPOLIA": {}});
        let v2 = json!({
            "version": 2,
            "accounts": {"SN_SEPOLIA": {}, "0x534e5f5345504f4c4941": {}}
        });

        for (input, expected_first, expected_second) in [
            (v1, "SN_SEPOLIA", "alpha-sepolia"),
            (v2, "0x534e5f5345504f4c4941", "SN_SEPOLIA"),
        ] {
            let input = serde_json::to_vec(&input).unwrap();
            let Err(AccountsFileError::DuplicateChainId {
                chain_id,
                first,
                second,
            }) = DecodedAccountRegistry::parse_json(&input)
            else {
                panic!("expected a duplicate chain ID error");
            };
            assert_eq!(chain_id.to_string(), "SN_SEPOLIA");
            assert_eq!(
                (first.as_str(), second.as_str()),
                (expected_first, expected_second)
            );
        }
    }
}
