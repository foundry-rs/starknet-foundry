use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;

use camino::{Utf8Path, Utf8PathBuf};
use fs2::FileExt;
use serde::Serialize;
use tempfile::Builder;

use crate::accounts::error::{FileType, StorageOperation};
use crate::accounts::schema::{DecodedAccountRegistry, SourceVersion};
use crate::accounts::{AccountName, AccountRecord, AccountRegistry, AccountsError, NetworkName};
use crate::helpers::filesystem::{ensure_regular_file, set_owner_only_permissions};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum MigrationOutcome {
    NotRequired {
        version: SourceVersion,
    },
    Performed {
        from: SourceVersion,
        to: SourceVersion,
        backup_path: Utf8PathBuf,
    },
}

#[derive(Debug)]
pub struct MutationResult<T> {
    pub value: T,
    pub migration_outcome: MigrationOutcome,
}

#[derive(Clone, Debug)]
pub struct AccountRepository {
    path: Utf8PathBuf,
}

impl AccountRepository {
    #[must_use]
    pub fn new(path: Utf8PathBuf) -> Result<Self, AccountsError> {
        ensure_regular_file(&path, false).map_err(map_storage_error(
            StorageOperation::Inspect,
            FileType::AccountsFile,
            &path,
        ))?;
        Ok(Self { path })
    }

    #[must_use]
    pub fn path(&self) -> &Utf8Path {
        &self.path
    }

    #[must_use]
    pub fn backup_path(&self, source_version: SourceVersion) -> Utf8PathBuf {
        path_with_suffix(&self.path, &format!(".{source_version}.bak"))
    }

    pub fn generate_account_name(&self) -> Result<String, AccountsError> {
        let mut id = 1;

        if !self.file_exists() {
            return Ok(format!("account-{id}"));
        }

        let used_ids: HashSet<u32> = self
            .load()?
            .registry
            .networks()
            .values()
            .flat_map(|accounts| accounts.keys())
            .filter_map(|name| {
                name.as_str()
                    .strip_prefix("account-")
                    .and_then(|id| id.parse::<u32>().ok())
            })
            .collect();

        while used_ids.contains(&id) {
            id += 1;
        }

        Ok(format!("account-{id}"))
    }

    pub fn load(&self) -> Result<DecodedAccountRegistry, AccountsError> {
        if !self.file_exists() {
            return Err(AccountsError::FileNotFound {
                path: self.path.clone(),
            });
        }
        DecodedAccountRegistry::parse_json(&self.read()?)
    }

    pub fn find(&self, network: &str, account: &str) -> Result<AccountRecord, AccountsError> {
        self.load()?
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
        network: NetworkName,
        name: AccountName,
        account: AccountRecord,
    ) -> Result<MutationResult<()>, AccountsError> {
        self.update(move |registry| {
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
        network: &str,
        name: &str,
    ) -> Result<MutationResult<AccountRecord>, AccountsError> {
        self.update(|registry| {
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

    pub fn update<T>(
        &self,
        operation: impl FnOnce(&mut AccountRegistry) -> Result<T, AccountsError>,
    ) -> Result<MutationResult<T>, AccountsError> {
        self.with_exclusive_lock(|| {
            let file_exists = self.file_exists();

            let original_content = if file_exists {
                self.read()?
            } else {
                Vec::new()
            };

            let DecodedAccountRegistry {
                mut registry,
                source_version,
            } = DecodedAccountRegistry::parse_json(&original_content)?;

            let value = operation(&mut registry)?;
            let encoded_content = registry.encode()?;

            let migration_outcome = if file_exists && !source_version.is_latest() {
                let backup_path = self.write_backup(&original_content, source_version)?;

                MigrationOutcome::Performed {
                    from: source_version,
                    to: SourceVersion::LATEST,
                    backup_path,
                }
            } else {
                MigrationOutcome::NotRequired {
                    version: source_version,
                }
            };

            self.write_atomic(&encoded_content)?;

            Ok(MutationResult {
                value,
                migration_outcome,
            })
        })
    }

    pub fn update_to_latest_schema(&self) -> Result<MigrationOutcome, AccountsError> {
        self.update(|_| Ok(()))
            .map(|result| result.migration_outcome)
    }

    pub fn file_exists(&self) -> bool {
        self.path.exists()
    }

    fn read(&self) -> Result<Vec<u8>, AccountsError> {
        fs::read(&self.path).map_err(map_storage_error(
            StorageOperation::Read,
            FileType::AccountsFile,
            &self.path,
        ))
    }

    fn write_atomic(&self, contents: &[u8]) -> Result<(), AccountsError> {
        let parent = ensure_parent(&self.path)?;

        let mut temporary = Builder::new()
            .prefix(".sncast-accounts-")
            .tempfile_in(parent)
            .map_err(map_storage_error(
                StorageOperation::Create,
                FileType::TemporaryAccountsFile,
                &self.path,
            ))?;

        set_owner_only_permissions(temporary.path()).map_err(map_storage_error(
            StorageOperation::SetPermissions,
            FileType::AccountsFile,
            &self.path,
        ))?;

        temporary.write_all(contents).map_err(map_storage_error(
            StorageOperation::Write,
            FileType::TemporaryAccountsFile,
            &self.path,
        ))?;

        temporary.flush().map_err(map_storage_error(
            StorageOperation::Flush,
            FileType::TemporaryAccountsFile,
            &self.path,
        ))?;

        temporary.as_file().sync_all().map_err(map_storage_error(
            StorageOperation::Sync,
            FileType::TemporaryAccountsFile,
            &self.path,
        ))?;

        temporary
            .persist(&self.path)
            .map_err(|error| error.error)
            .map_err(map_storage_error(
                StorageOperation::Replace,
                FileType::AccountsFile,
                &self.path,
            ))?;

        sync_parent(parent, &self.path)
    }

    fn write_backup(
        &self,
        contents: &[u8],
        source_version: SourceVersion,
    ) -> Result<Utf8PathBuf, AccountsError> {
        let path = self.backup_path(source_version);

        ensure_regular_file(&path.to_path_buf(), false).map_err(map_storage_error(
            StorageOperation::Inspect,
            FileType::AccountsFile,
            &path,
        ))?;

        let parent = ensure_parent(&path)?;

        let mut options = OpenOptions::new();
        options.write(true).create(true).mode(0o600);

        let mut file = options.open(&path).map_err(map_storage_error(
            StorageOperation::Create,
            FileType::Backup,
            &path,
        ))?;

        file.write_all(contents).map_err(map_storage_error(
            StorageOperation::Write,
            FileType::Backup,
            &path,
        ))?;

        file.sync_all().map_err(map_storage_error(
            StorageOperation::Sync,
            FileType::Backup,
            &path,
        ))?;

        sync_parent(parent, &path)?;

        Ok(path)
    }

    fn with_exclusive_lock<T>(
        &self,
        operation: impl FnOnce() -> Result<T, AccountsError>,
    ) -> Result<T, AccountsError> {
        let parent = ensure_parent(&self.path)?;
        let lock_path = path_with_suffix(&self.path, ".lock");

        ensure_regular_file(&lock_path, false).map_err(map_storage_error(
            StorageOperation::Inspect,
            FileType::AccountsLockFile,
            &lock_path,
        ))?;

        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).mode(0o600);

        let lock = options.open(&lock_path).map_err(map_storage_error(
            StorageOperation::Read,
            FileType::AccountsLockFile,
            &lock_path,
        ))?;

        lock.lock_exclusive().map_err(map_storage_error(
            StorageOperation::Lock,
            FileType::AccountsFile,
            &self.path,
        ))?;

        let result = operation();

        FileExt::unlock(&lock).map_err(map_storage_error(
            StorageOperation::Unlock,
            FileType::AccountsFile,
            &self.path,
        ))?;

        sync_parent(parent, &self.path)?;

        result
    }
}

fn path_with_suffix(path: &Utf8Path, suffix: &str) -> Utf8PathBuf {
    let file_name = path.file_name().expect("regular file should have a name");
    path.with_file_name(format!("{file_name}{suffix}"))
}

fn ensure_parent(path: &Utf8Path) -> Result<&Utf8Path, AccountsError> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_str().is_empty())
        .unwrap_or(Utf8Path::new("."));

    fs::create_dir_all(parent).map_err(map_storage_error(
        StorageOperation::Create,
        FileType::ParentDirectory,
        parent,
    ))?;

    Ok(parent)
}

fn sync_parent(parent: &Utf8Path, accounts_path: &Utf8Path) -> Result<(), AccountsError> {
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(map_storage_error(
            StorageOperation::Sync,
            FileType::ParentDirectory,
            accounts_path,
        ))
}

fn map_storage_error(
    operation: StorageOperation,
    file_type: FileType,
    path: &Utf8Path,
) -> impl FnOnce(io::Error) -> AccountsError {
    move |source| AccountsError::Storage {
        operation,
        file_type,
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{Arc, Barrier};
    use std::thread;

    use tempfile::tempdir;

    use super::*;
    use crate::accounts::AccountsFileError;
    use crate::signers::{PrivateKeySpec, SignerSpec};
    use starknet_types_core::felt::Felt;

    const V1_ACCOUNT: &str = r#"{
        "alpha-sepolia": {
            "alice": {
                "public_key": "0x1",
                "private_key": "0x2",
                "address": "0x3",
                "deployed": false
            }
        }
    }"#;

    fn path() -> (tempfile::TempDir, Utf8PathBuf) {
        let directory = tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(directory.path().join("accounts.json")).unwrap();
        (directory, path)
    }

    fn account(address: Felt) -> AccountRecord {
        AccountRecord {
            public_key: Felt::ONE,
            address,
            salt: None,
            deployed: None,
            class_hash: None,
            legacy: None,
            account_type: None,
            signer: SignerSpec::PrivateKey(PrivateKeySpec::new(Felt::TWO)),
        }
    }

    #[test]
    fn read_only_load_does_not_migrate_v1() {
        let (_directory, path) = path();
        fs::write(&path, V1_ACCOUNT).unwrap();

        let repository = AccountRepository::new(path.clone()).unwrap();
        let decoded = repository.load().unwrap();

        assert_eq!(decoded.source_version, SourceVersion::V1);
        assert_eq!(fs::read_to_string(&path).unwrap(), V1_ACCOUNT);
        assert!(!repository.backup_path(SourceVersion::V1).exists());
    }

    #[test]
    fn reports_missing_file() {
        let (_directory, path) = path();

        assert!(!AccountRepository::new(path).unwrap().file_exists());
    }

    #[test]
    fn uses_current_directory_for_relative_accounts_path() {
        assert_eq!(
            ensure_parent(Utf8Path::new("accounts.json")).unwrap(),
            Utf8Path::new(".")
        );
    }

    #[test]
    fn generates_first_account_name_when_file_is_missing() {
        let (_directory, path) = path();

        assert_eq!(
            AccountRepository::new(path)
                .unwrap()
                .generate_account_name()
                .unwrap(),
            "account-1"
        );
    }

    #[test]
    fn generates_next_account_name_from_non_empty_file() {
        let (_directory, path) = path();
        let repository = AccountRepository::new(path).unwrap();
        for name in ["account-1", "account-2"] {
            repository
                .insert(
                    NetworkName::new("alpha-sepolia").unwrap(),
                    AccountName::new(name).unwrap(),
                    account(Felt::ONE),
                )
                .unwrap();
        }

        assert_eq!(repository.generate_account_name().unwrap(), "account-3");
    }

    #[test]
    fn finds_existing_account_and_reports_missing_account() {
        let (_directory, path) = path();
        let repository = AccountRepository::new(path).unwrap();
        repository
            .insert(
                NetworkName::new("alpha-sepolia").unwrap(),
                AccountName::new("alice").unwrap(),
                account(Felt::THREE),
            )
            .unwrap();

        assert_eq!(
            repository.find("alpha-sepolia", "alice").unwrap().address,
            Felt::THREE
        );
        assert!(matches!(
            repository.find("alpha-sepolia", "bob"),
            Err(AccountsError::AccountNotFound { network, account })
                if network == "alpha-sepolia" && account == "bob"
        ));
    }

    #[test]
    fn removes_existing_account_and_reports_missing_account() {
        let (_directory, path) = path();
        let repository = AccountRepository::new(path).unwrap();
        repository
            .insert(
                NetworkName::new("alpha-sepolia").unwrap(),
                AccountName::new("alice").unwrap(),
                account(Felt::THREE),
            )
            .unwrap();

        assert_eq!(
            repository
                .remove("alpha-sepolia", "alice")
                .unwrap()
                .value
                .address,
            Felt::THREE
        );
        assert!(matches!(
            repository.remove("alpha-sepolia", "alice"),
            Err(AccountsError::AccountNotFound { network, account })
                if network == "alpha-sepolia" && account == "alice"
        ));
    }

    #[test]
    fn successful_v1_mutation_writes_v2_and_backup() {
        let (_directory, path) = path();
        fs::write(&path, V1_ACCOUNT).unwrap();

        let repository = AccountRepository::new(path.clone()).unwrap();
        let result = repository
            .update(|registry| {
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

        assert_eq!(
            result.migration_outcome,
            MigrationOutcome::Performed {
                from: SourceVersion::V1,
                to: SourceVersion::V2,
                backup_path: repository.backup_path(SourceVersion::V1),
            }
        );
        assert_eq!(
            fs::read_to_string(repository.backup_path(SourceVersion::V1)).unwrap(),
            V1_ACCOUNT
        );
        let decoded = repository.load().unwrap();
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

        let repository = AccountRepository::new(path.clone()).unwrap();
        let result = repository.update(|_| {
            Err::<(), _>(AccountsError::MissingField {
                field: "address",
                operation: "test",
            })
        });

        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), V1_ACCOUNT);
        assert!(!repository.backup_path(SourceVersion::V1).exists());
    }

    #[test]
    fn schema_errors_include_field_paths() {
        let (_directory, path) = path();
        fs::write(
            &path,
            r#"{"alpha-sepolia":{"alice":{"public_key":"not-a-felt","private_key":"0x1"}}}"#,
        )
        .unwrap();

        let error = AccountRepository::new(path.clone())
            .unwrap()
            .load()
            .unwrap_err();

        assert!(matches!(
            error,
            AccountsError::AccountsFile(AccountsFileError::Schema { field_path, source: _ })
            if field_path  == "alpha-sepolia.alice.public_key"
        ));
    }

    #[test]
    fn inserts_new_accounts_as_v2() {
        let (_directory, path) = path();
        let result = AccountRepository::new(path.clone())
            .unwrap()
            .insert(
                NetworkName::new("alpha-sepolia").unwrap(),
                AccountName::new("alice").unwrap(),
                account(Felt::TWO),
            )
            .unwrap();

        assert_eq!(
            result.migration_outcome,
            MigrationOutcome::NotRequired {
                version: SourceVersion::V2,
            }
        );
        assert_eq!(
            AccountRepository::new(path.clone())
                .unwrap()
                .load()
                .unwrap()
                .source_version,
            SourceVersion::V2
        );
    }

    #[test]
    fn reports_v2_migration_not_required_for_existing_file() {
        let (_directory, path) = path();
        let repository = AccountRepository::new(path).unwrap();
        repository
            .insert(
                NetworkName::new("alpha-sepolia").unwrap(),
                AccountName::new("alice").unwrap(),
                account(Felt::TWO),
            )
            .unwrap();

        let result = repository.update(|_| Ok(())).unwrap();

        assert_eq!(
            result.migration_outcome,
            MigrationOutcome::NotRequired {
                version: SourceVersion::V2,
            }
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
                    AccountRepository::new(path)
                        .unwrap()
                        .insert(
                            NetworkName::new("alpha-sepolia").unwrap(),
                            AccountName::new(format!("account-{index}")).unwrap(),
                            account(Felt::from(index + 100)),
                        )
                        .unwrap();
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap();
        }

        let decoded = AccountRepository::new(path.clone())
            .unwrap()
            .load()
            .unwrap();
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

    #[test]
    fn writes_complete_file_with_secret_permissions() {
        let (_directory, path) = path();

        AccountRepository::new(path.clone())
            .unwrap()
            .write_atomic(b"{\"version\":2}")
            .unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"{\"version\":2}");

        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[test]
    fn rejects_accounts_file_symlinks() {
        use std::os::unix::fs::symlink;

        let (directory, path) = path();
        let target = directory.path().join("target.json");
        fs::write(&target, "{}").unwrap();
        symlink(&target, &path).unwrap();

        assert!(matches!(
            AccountRepository::new(path).unwrap().read(),
            Err(AccountsError::Storage {
                operation: StorageOperation::Inspect,
                source,
                ..
            }) if source.kind() == std::io::ErrorKind::PermissionDenied
        ));
    }
}
