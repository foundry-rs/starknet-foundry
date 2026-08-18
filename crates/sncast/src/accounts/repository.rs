use camino::{Utf8Path, Utf8PathBuf};

use crate::accounts::schema::{AccountsCodec, DecodedRegistry, SourceVersion};
use crate::accounts::storage::{AccountsStorage, FileSystemAccountsStorage};
use crate::accounts::{AccountName, AccountRecord, AccountRegistry, AccountsError, NetworkName};

#[derive(Debug)]
pub struct MutationResult<T> {
    pub value: T,
    pub migrated_from_v1: bool,
}

#[derive(Clone, Debug)]
pub struct AccountRepository<S = FileSystemAccountsStorage> {
    storage: S,
    codec: AccountsCodec,
}

impl Default for AccountRepository<FileSystemAccountsStorage> {
    fn default() -> Self {
        Self::new(FileSystemAccountsStorage)
    }
}

impl<S: AccountsStorage> AccountRepository<S> {
    #[must_use]
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            codec: AccountsCodec,
        }
    }

    pub fn load(&self, path: &Utf8Path) -> Result<DecodedRegistry, AccountsError> {
        if !self.storage.exists(path)? {
            return Err(AccountsError::FileNotFound {
                path: path.to_owned(),
            });
        }
        self.codec.decode(&self.storage.read(path)?)
    }

    pub fn find(
        &self,
        path: &Utf8Path,
        network: &str,
        account: &str,
    ) -> Result<AccountRecord, AccountsError> {
        self.load(path)?
            .registry
            .account(network, account)
            .cloned()
            .ok_or_else(|| AccountsError::AccountNotFound {
                network: network.to_owned(),
                account: account.to_owned(),
            })
    }

    pub fn insert(
        &self,
        path: &Utf8Path,
        network: NetworkName,
        name: AccountName,
        account: AccountRecord,
    ) -> Result<MutationResult<()>, AccountsError> {
        self.mutate(path, move |registry| {
            let network_display = network.to_string();
            let name_display = name.to_string();
            let existing = registry
                .networks_mut()
                .entry(network)
                .or_default()
                .insert(name, account);
            if existing.is_some() {
                return Err(AccountsError::DuplicateAccount {
                    network: network_display,
                    account: name_display,
                });
            }
            Ok(())
        })
    }

    pub fn remove(
        &self,
        path: &Utf8Path,
        network: &str,
        name: &str,
    ) -> Result<MutationResult<AccountRecord>, AccountsError> {
        self.mutate(path, |registry| {
            registry
                .networks_mut()
                .get_mut(network)
                .and_then(|accounts| accounts.remove(name))
                .ok_or_else(|| AccountsError::AccountNotFound {
                    network: network.to_owned(),
                    account: name.to_owned(),
                })
        })
    }

    pub fn mutate<T>(
        &self,
        path: &Utf8Path,
        operation: impl FnOnce(&mut AccountRegistry) -> Result<T, AccountsError>,
    ) -> Result<MutationResult<T>, AccountsError> {
        self.storage.with_exclusive_lock(path, || {
            let existed = self.storage.exists(path)?;
            let original = if existed {
                self.storage.read(path)?
            } else {
                Vec::new()
            };
            let mut decoded = self.codec.decode(&original)?;
            let value = operation(&mut decoded.registry)?;
            let encoded = self.codec.encode_v2(&decoded.registry)?;
            let migrated_from_v1 = existed && decoded.source_version == SourceVersion::V1;

            if migrated_from_v1 {
                self.storage
                    .write_backup_if_absent(&v1_backup_path(path), &original)?;
            }
            self.storage.write_atomic(path, &encoded)?;

            Ok(MutationResult {
                value,
                migrated_from_v1,
            })
        })
    }
}

#[must_use]
pub fn v1_backup_path(path: &Utf8Path) -> Utf8PathBuf {
    let file_name = path.file_name().unwrap_or("accounts.json");
    path.with_file_name(format!("{file_name}.v1.bak"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::{Arc, Barrier};
    use std::thread;

    use tempfile::tempdir;

    use super::*;
    use crate::signers::{PrivateKeySpec, SignerSpec};
    use starknet_types_core::felt::Felt;

    const V1_ACCOUNT: &str = r#"{
        "alpha-sepolia": {
            "alice": {
                "public_key": "0x1",
                "private_key": "0x2",
                "deployed": false
            }
        }
    }"#;

    fn path() -> (tempfile::TempDir, Utf8PathBuf) {
        let directory = tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(directory.path().join("accounts.json")).unwrap();
        (directory, path)
    }

    #[test]
    fn read_only_load_does_not_migrate_v1() {
        let (_directory, path) = path();
        fs::write(&path, V1_ACCOUNT).unwrap();

        let decoded = AccountRepository::default().load(&path).unwrap();

        assert_eq!(decoded.source_version, SourceVersion::V1);
        assert_eq!(fs::read_to_string(&path).unwrap(), V1_ACCOUNT);
        assert!(!v1_backup_path(&path).exists());
    }

    #[test]
    fn successful_v1_mutation_writes_v2_and_backup() {
        let (_directory, path) = path();
        fs::write(&path, V1_ACCOUNT).unwrap();

        let result = AccountRepository::default()
            .mutate(&path, |registry| {
                registry
                    .networks_mut()
                    .get_mut("alpha-sepolia")
                    .unwrap()
                    .get_mut("alice")
                    .unwrap()
                    .deployed = Some(true);
                Ok(())
            })
            .unwrap();

        assert!(result.migrated_from_v1);
        assert_eq!(
            fs::read_to_string(v1_backup_path(&path)).unwrap(),
            V1_ACCOUNT
        );
        let decoded = AccountRepository::default().load(&path).unwrap();
        assert_eq!(decoded.source_version, SourceVersion::V2);
        assert_eq!(
            decoded
                .registry
                .account("alpha-sepolia", "alice")
                .unwrap()
                .deployed,
            Some(true)
        );
    }

    #[test]
    fn failed_mutation_does_not_write_or_backup() {
        let (_directory, path) = path();
        fs::write(&path, V1_ACCOUNT).unwrap();

        let result = AccountRepository::default().mutate(&path, |_| {
            Err::<(), _>(AccountsError::MissingField {
                field: "address",
                operation: "test",
            })
        });

        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), V1_ACCOUNT);
        assert!(!v1_backup_path(&path).exists());
    }

    #[test]
    fn inserts_new_accounts_as_v2() {
        let (_directory, path) = path();
        let account = AccountRecord {
            public_key: Felt::ONE,
            address: None,
            salt: None,
            deployed: None,
            class_hash: None,
            legacy: None,
            account_type: None,
            signer: SignerSpec::PrivateKey(PrivateKeySpec::new(Felt::TWO)),
        };

        AccountRepository::default()
            .insert(
                &path,
                NetworkName::new("alpha-sepolia").unwrap(),
                AccountName::new("alice").unwrap(),
                account,
            )
            .unwrap();

        assert_eq!(
            AccountRepository::default()
                .load(&path)
                .unwrap()
                .source_version,
            SourceVersion::V2
        );
    }

    #[test]
    fn concurrent_mutations_do_not_lose_accounts() {
        const WRITERS: usize = 8;
        let (_directory, path) = path();
        let barrier = Arc::new(Barrier::new(WRITERS));
        let handles = (0..WRITERS)
            .map(|index| {
                let barrier = Arc::clone(&barrier);
                let path = path.clone();
                thread::spawn(move || {
                    barrier.wait();
                    AccountRepository::default()
                        .insert(
                            &path,
                            NetworkName::new("alpha-sepolia").unwrap(),
                            AccountName::new(format!("account-{index}")).unwrap(),
                            AccountRecord {
                                public_key: Felt::from(index + 1),
                                address: None,
                                salt: None,
                                deployed: Some(false),
                                class_hash: None,
                                legacy: None,
                                account_type: None,
                                signer: SignerSpec::PrivateKey(PrivateKeySpec::new(Felt::from(
                                    index + 10,
                                ))),
                            },
                        )
                        .unwrap();
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap();
        }

        let decoded = AccountRepository::default().load(&path).unwrap();
        assert_eq!(
            decoded
                .registry
                .networks()
                .get("alpha-sepolia")
                .unwrap()
                .len(),
            WRITERS
        );
    }
}
