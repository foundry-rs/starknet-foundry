use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::accounts::{AccountRegistry, AccountsError};

pub mod migration;
pub mod v1;
pub mod v2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceVersion {
    V1,
    V2,
}

#[derive(Debug)]
pub enum VersionedAccountsFile {
    V1(v1::AccountsFile),
    V2(v2::AccountsFile),
}

impl VersionedAccountsFile {
    pub fn decode(input: &[u8]) -> Result<Self, AccountsError> {
        if input.iter().all(u8::is_ascii_whitespace) {
            return Ok(Self::V1(v1::AccountsFile::default()));
        }

        let envelope: Value = deserialize(input)?;
        match envelope.get("version") {
            None => deserialize(input).map(Self::V1),
            Some(Value::Number(version)) if version.as_u64() == Some(u64::from(v2::VERSION)) => {
                let file: v2::AccountsFile = deserialize(input)?;
                if file.version != v2::VERSION {
                    return Err(AccountsError::UnsupportedVersion {
                        version: file.version.to_string(),
                    });
                }
                Ok(Self::V2(file))
            }
            Some(version) => Err(AccountsError::UnsupportedVersion {
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
    pub fn decode(input: &[u8]) -> Result<Self, AccountsError> {
        match VersionedAccountsFile::decode(input)? {
            VersionedAccountsFile::V1(file) => Ok(Self {
                registry: file.try_into()?,
                source_version: SourceVersion::V1,
            }),
            VersionedAccountsFile::V2(file) => Ok(Self {
                registry: file.try_into()?,
                source_version: SourceVersion::V2,
            }),
        }
    }

    pub fn encode_v2(registry: &AccountRegistry) -> Result<Vec<u8>, AccountsError> {
        let file: v2::AccountsFile = registry.try_into()?;
        let mut output = serde_json::to_vec_pretty(&file).map_err(|error| schema_error(&error))?;
        output.push(b'\n');
        Ok(output)
    }
}

fn deserialize<T: DeserializeOwned>(input: &[u8]) -> Result<T, AccountsError> {
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    serde_path_to_error::deserialize(&mut deserializer).map_err(|error| AccountsError::Schema {
        path: error.path().to_string(),
        message: error.into_inner().to_string(),
    })
}

fn schema_error(error: &serde_json::Error) -> AccountsError {
    AccountsError::Schema {
        path: String::new(),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use camino::Utf8PathBuf;
    use serde_json::json;
    use starknet_types_core::felt::Felt;

    use super::*;
    use crate::accounts::{AccountName, AccountRecord, AccountType, NetworkName};
    use crate::signers::{KeystoreSpec, SignerSpec};

    #[test]
    fn decodes_existing_accounts_fixture_as_v1() {
        let decoded = DecodedAccountRegistry::decode(include_bytes!(
            "../../../tests/data/accounts/accounts.json"
        ))
        .unwrap();

        assert_eq!(decoded.source_version, SourceVersion::V1);
        assert!(decoded.registry.account("alpha-sepolia", "user1").is_some());
    }

    #[test]
    fn preserves_v1_untagged_precedence_only_in_v1() {
        let input = br#"{
            "alpha-sepolia": {
                "alice": {
                    "public_key": "0x1",
                    "private_key": "0x2",
                    "ledger_path": "m/44'/60'/0'/0/0"
                }
            }
        }"#;

        let decoded = DecodedAccountRegistry::decode(input).unwrap();
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
                        "signer": {
                            "type": "keystore",
                            "path": "keys/alice.json",
                            "password_env": "ALICE_PASSWORD"
                        }
                    }
                }
            }
        }"#;

        let decoded = DecodedAccountRegistry::decode(input).unwrap();
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
    fn rejects_unknown_version_and_untagged_v2_signer() {
        let unknown_version = serde_json::to_vec(&json!({"version": 3, "accounts": {}})).unwrap();
        assert!(matches!(
            DecodedAccountRegistry::decode(&unknown_version),
            Err(AccountsError::UnsupportedVersion { .. })
        ));

        let untagged = br#"{
            "version": 2,
            "accounts": {
                "alpha-sepolia": {
                    "alice": {"public_key": "0x1", "private_key": "0x2"}
                }
            }
        }"#;
        assert!(matches!(
            DecodedAccountRegistry::decode(untagged),
            Err(AccountsError::Schema { .. })
        ));
    }

    #[test]
    fn encodes_only_deterministic_v2() {
        let account = AccountRecord {
            public_key: Felt::ONE,
            address: None,
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
        let registry = AccountRegistry::new(std::collections::BTreeMap::from([(
            NetworkName::new("alpha-sepolia").unwrap(),
            std::collections::BTreeMap::from([(AccountName::new("alice").unwrap(), account)]),
        )]));

        let encoded = DecodedAccountRegistry::encode_v2(&registry).unwrap();
        let value: Value = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(value["version"], 2);
        assert_eq!(
            value["accounts"]["alpha-sepolia"]["alice"]["signer"]["type"],
            "keystore"
        );
        assert_eq!(
            encoded,
            DecodedAccountRegistry::encode_v2(&registry).unwrap()
        );
    }
}
