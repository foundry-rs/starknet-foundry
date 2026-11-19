use std::collections::HashSet;
use std::collections::btree_map::Entry;
use std::io;

use camino::{Utf8Path, Utf8PathBuf};
use foundry_ui::UI;
use serde::Serialize;

use crate::accounts::error::{FileType, StorageOperation};
use crate::accounts::schema::{DecodedAccountRegistry, SourceVersion};
use crate::accounts::{AccountName, AccountRecord, AccountRegistry, AccountsError, ChainId};
use crate::helpers::filesystem::{self, LockableFile, NonMaterialized};
use crate::response::ui::UIExt;

pub struct AccountRepository {
    accounts_file: LockableFile<NonMaterialized>,
}

impl AccountRepository {
    pub fn try_new(accounts_file_path: &Utf8Path) -> Result<Self, AccountsError> {
        let accounts_file = LockableFile::try_new(accounts_file_path).map_to_accounts_error(
            StorageOperation::Inspect,
            FileType::AccountsFile,
            accounts_file_path,
        )?;

        Ok(Self { accounts_file })
    }

    pub fn find(
        &self,
        chain_id: &ChainId,
        name: &AccountName,
    ) -> Result<AccountRecord, AccountsError> {
        self.load()?
            .registry
            .account(chain_id, name)
            .cloned()
            .ok_or_else(|| AccountsError::AccountNotFound {
                chain_id: *chain_id,
                account: name.clone(),
            })
    }

    /// Inserts a record under `name`, creating the accounts file if it does not exist.
    pub fn insert(
        &self,
        chain_id: ChainId,
        name: AccountName,
        record: AccountRecord,
    ) -> Result<MutationResult<()>, AccountsError> {
        self.update(Mutation::Insert, move |registry| {
            insert_into(registry, chain_id, name, record)
        })
    }

    /// Inserts a record under the first free `account-<n>` name and returns that name.
    ///
    /// Choosing the name and inserting happen under one writer lock, so concurrent callers
    /// never pick the same name. Names are unique across all chains.
    pub fn insert_with_generated_name(
        &self,
        chain_id: ChainId,
        record: AccountRecord,
    ) -> Result<MutationResult<AccountName>, AccountsError> {
        self.update(Mutation::Insert, move |registry| {
            let name = generate_account_name(registry);
            insert_into(registry, chain_id, name.clone(), record)?;
            Ok(name)
        })
    }

    pub fn remove(
        &self,
        chain_id: &ChainId,
        name: &AccountName,
    ) -> Result<MutationResult<AccountRecord>, AccountsError> {
        self.update(Mutation::Modify, |registry| {
            registry
                .networks_mut()
                .get_mut(chain_id)
                .and_then(|accounts| accounts.remove(name))
                .ok_or_else(|| AccountsError::AccountNotFound {
                    chain_id: *chain_id,
                    account: name.clone(),
                })
        })
    }

    pub fn update_record(
        &self,
        chain_id: &ChainId,
        name: &AccountName,
        operation: impl FnOnce(&mut AccountRecord) -> Result<(), AccountsError>,
    ) -> Result<MutationResult<()>, AccountsError> {
        self.update(Mutation::Modify, |registry| {
            registry
                .networks_mut()
                .get_mut(chain_id)
                .and_then(|accounts| accounts.get_mut(name))
                .ok_or_else(|| AccountsError::AccountNotFound {
                    chain_id: *chain_id,
                    account: name.clone(),
                })
                .and_then(operation)
        })
    }

    pub fn update_to_latest_schema(&self) -> Result<MigrationOutcome, AccountsError> {
        self.update(Mutation::Migrate, |_| Ok(()))
            .map(|result| result.migration_outcome)
    }

    #[must_use]
    pub fn path(&self) -> &Utf8Path {
        self.accounts_file.path()
    }

    #[must_use]
    pub fn file_exists(&self) -> bool {
        self.accounts_file.path().exists()
    }

    #[must_use]
    pub fn backup_path(&self, source_version: SourceVersion) -> Utf8PathBuf {
        let path = self.accounts_file.path();
        let file_name = path.file_name().expect("regular file should have a name");
        path.with_file_name(format!("{file_name}.{source_version}.bak"))
    }

    /// Reads the accounts file without locking it; see [`LockableFile::read_if_exists`].
    ///
    /// A missing file is [`AccountsError::FileNotFound`]; an existing blank file is an empty registry.
    pub fn load(&self) -> Result<DecodedAccountRegistry, AccountsError> {
        let contents = self
            .accounts_file
            .read_if_exists()
            .map_to_accounts_error(
                StorageOperation::Read,
                FileType::AccountsFile,
                self.accounts_file.path(),
            )?
            .ok_or_else(|| self.file_not_found())?;

        self.decode(&contents)
    }

    fn file_not_found(&self) -> AccountsError {
        AccountsError::FileNotFound {
            path: self.accounts_file.path().to_owned(),
        }
    }

    fn decode(&self, contents: &[u8]) -> Result<DecodedAccountRegistry, AccountsError> {
        DecodedAccountRegistry::parse_json(contents).map_err(|source| {
            AccountsError::InvalidAccountsFile {
                path: self.accounts_file.path().to_owned(),
                source,
            }
        })
    }

    fn update<T>(
        &self,
        mutation: Mutation,
        operation: impl FnOnce(&mut AccountRegistry) -> Result<T, AccountsError>,
    ) -> Result<MutationResult<T>, AccountsError> {
        // Only insertion may create the file; other mutations need existing data.
        if !matches!(mutation, Mutation::Insert) && !self.file_exists() {
            return Err(self.file_not_found());
        }

        let accounts_file = self.accounts_file.materialize().map_to_accounts_error(
            StorageOperation::Create,
            FileType::AccountsFile,
            self.accounts_file.path(),
        )?;

        let locked_file = accounts_file.lock().map_to_accounts_error(
            StorageOperation::Lock,
            FileType::AccountsFile,
            self.accounts_file.path(),
        )?;

        let original_contents = locked_file.read().map_to_accounts_error(
            StorageOperation::Read,
            FileType::AccountsFile,
            self.accounts_file.path(),
        )?;

        let DecodedAccountRegistry {
            mut registry,
            source_version,
        } = self.decode(&original_contents)?;

        let mutation_outcome = operation(&mut registry)?;

        // Blank files, including ones just created by `materialize`, decode as v1 but hold nothing to migrate.
        let needs_backup =
            !source_version.is_latest() && !original_contents.trim_ascii().is_empty();

        let migration_outcome = if needs_backup {
            let backup_path = self.write_backup(&original_contents, source_version)?;
            MigrationOutcome::Performed {
                from: source_version,
                to: SourceVersion::LATEST,
                backup_path,
            }
        } else {
            MigrationOutcome::NotRequired {
                version: SourceVersion::LATEST,
            }
        };

        if !matches!(mutation, Mutation::Migrate) || !source_version.is_latest() {
            locked_file
                .write_atomic(&registry.encode()?)
                .map_to_accounts_error(
                    StorageOperation::Write,
                    FileType::AccountsFile,
                    self.accounts_file.path(),
                )?;
        }

        Ok(MutationResult {
            value: mutation_outcome,
            migration_outcome,
        })
    }

    fn write_backup(
        &self,
        contents: &[u8],
        source_version: SourceVersion,
    ) -> Result<Utf8PathBuf, AccountsError> {
        let path = self.backup_path(source_version);

        filesystem::write_atomic(&path, contents).map_to_accounts_error(
            StorageOperation::Write,
            FileType::Backup,
            &path,
        )?;

        Ok(path)
    }
}

#[derive(Clone, Copy)]
enum Mutation {
    /// Adds an account; creates the accounts file when it is missing.
    Insert,
    /// Changes existing data; always rewrites the file.
    Modify,
    /// Rewrites the file only when its schema is outdated.
    Migrate,
}

fn insert_into(
    registry: &mut AccountRegistry,
    chain_id: ChainId,
    name: AccountName,
    record: AccountRecord,
) -> Result<(), AccountsError> {
    match registry
        .networks_mut()
        .entry(chain_id)
        .or_default()
        .entry(name)
    {
        Entry::Occupied(entry) => Err(AccountsError::DuplicateAccount {
            chain_id,
            account: entry.key().clone(),
        }),
        Entry::Vacant(entry) => {
            entry.insert(record);
            Ok(())
        }
    }
}

/// Returns `account-<n>` with the smallest `n` not used by an account on any chain.
fn generate_account_name(registry: &AccountRegistry) -> AccountName {
    let used_ids: HashSet<u32> = registry
        .networks()
        .values()
        .flat_map(|accounts| accounts.keys())
        .filter_map(|name| {
            name.as_str()
                .strip_prefix("account-")
                .and_then(|id| id.parse::<u32>().ok())
        })
        .collect();

    let mut id = 1;
    while used_ids.contains(&id) {
        id += 1;
    }

    AccountName::new(format!("account-{id}")).expect("generated account name is not empty")
}

#[derive(Debug)]
pub struct MutationResult<T> {
    pub value: T,
    pub migration_outcome: MigrationOutcome,
}

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

impl MigrationOutcome {
    pub fn notify_if_migrated(&self, ui: &UI) {
        if let MigrationOutcome::Performed {
            from,
            to,
            backup_path,
        } = self
        {
            ui.print_warning(
                format!("Accounts file was migrated from schema version {from} to {to}; a backup of the original was saved at {backup_path}"),
            );
        }
    }
}

trait MapStorageError<T> {
    fn map_to_accounts_error(
        self,
        operation: StorageOperation,
        file_type: FileType,
        path: &Utf8Path,
    ) -> Result<T, AccountsError>;
}

impl<T> MapStorageError<T> for io::Result<T> {
    fn map_to_accounts_error(
        self,
        operation: StorageOperation,
        file_type: FileType,
        path: &Utf8Path,
    ) -> Result<T, AccountsError> {
        self.map_err(move |source| AccountsError::Storage {
            operation,
            file_type,
            path: path.to_owned(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use starknet_types_core::felt::Felt;
    use tempfile::tempdir;

    use super::*;
    use crate::signers::{PrivateKeySpec, SignerSpec};

    #[test]
    fn mutations_persist_but_current_schema_migration_preserves_bytes() {
        let directory = tempdir().unwrap();
        let path = Utf8PathBuf::try_from(directory.path().join("accounts.json")).unwrap();
        let repository = AccountRepository::try_new(&path).unwrap();

        let result = repository
            .insert(
                "SN_SEPOLIA".parse().unwrap(),
                AccountName::new("alice").unwrap(),
                AccountRecord {
                    public_key: Felt::ONE,
                    address: Felt::TWO,
                    salt: None,
                    deployed: None,
                    class_hash: None,
                    legacy: None,
                    account_type: None,
                    signer: SignerSpec::PrivateKey(PrivateKeySpec::new(Felt::THREE)),
                },
            )
            .unwrap();

        assert_eq!(
            result.migration_outcome,
            MigrationOutcome::NotRequired {
                version: SourceVersion::LATEST
            }
        );
        assert!(!repository.backup_path(SourceVersion::V1).exists());

        let original = format!("\n {} \n", std::fs::read_to_string(&path).unwrap());
        std::fs::write(&path, &original).unwrap();
        assert_eq!(
            repository.update_to_latest_schema().unwrap(),
            MigrationOutcome::NotRequired {
                version: SourceVersion::LATEST
            }
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
        assert!(!repository.backup_path(SourceVersion::V1).exists());

        let chain_id = "SN_SEPOLIA".parse().unwrap();
        let name = AccountName::new("alice").unwrap();
        repository
            .update_record(&chain_id, &name, |account| {
                account.salt = Some(Felt::from(17));
                Ok(())
            })
            .unwrap();
        assert_eq!(
            repository.find(&chain_id, &name).unwrap().salt,
            Some(Felt::from(17))
        );
    }

    fn record(address: u64) -> AccountRecord {
        AccountRecord {
            public_key: Felt::ONE,
            address: Felt::from(address),
            salt: None,
            deployed: None,
            class_hash: None,
            legacy: None,
            account_type: None,
            signer: SignerSpec::PrivateKey(PrivateKeySpec::new(Felt::THREE)),
        }
    }

    fn accounts_path(directory: &tempfile::TempDir) -> Utf8PathBuf {
        Utf8PathBuf::try_from(directory.path().join("accounts.json")).unwrap()
    }

    #[test]
    fn required_operations_reject_missing_file_without_creating_it() {
        let directory = tempdir().unwrap();
        let path = accounts_path(&directory);
        let repository = AccountRepository::try_new(&path).unwrap();
        let chain_id: ChainId = "SN_SEPOLIA".parse().unwrap();
        let name = AccountName::new("alice").unwrap();

        let results = [
            repository.load().map(|_| ()),
            repository.find(&chain_id, &name).map(|_| ()),
            repository.remove(&chain_id, &name).map(|_| ()),
            repository
                .update_record(&chain_id, &name, |_| Ok(()))
                .map(|_| ()),
            repository.update_to_latest_schema().map(|_| ()),
        ];

        for result in results {
            assert!(
                matches!(&result, Err(AccountsError::FileNotFound { path: missing }) if *missing == path),
                "{result:?}"
            );
        }
        assert!(!path.exists());
        assert!(!Utf8PathBuf::from(format!("{path}.lock")).exists());
    }

    #[test]
    fn blank_file_is_empty_registry_and_malformed_file_is_invalid() {
        let directory = tempdir().unwrap();
        let path = accounts_path(&directory);
        let repository = AccountRepository::try_new(&path).unwrap();
        let chain_id: ChainId = "SN_SEPOLIA".parse().unwrap();
        let name = AccountName::new("alice").unwrap();

        std::fs::write(&path, " \n").unwrap();
        assert!(repository.load().unwrap().registry.networks().is_empty());
        assert!(matches!(
            repository.find(&chain_id, &name),
            Err(AccountsError::AccountNotFound { .. })
        ));

        std::fs::write(&path, "{").unwrap();
        assert!(matches!(
            repository.load(),
            Err(AccountsError::InvalidAccountsFile { .. })
        ));
        assert!(matches!(
            repository.insert_with_generated_name(chain_id, record(1)),
            Err(AccountsError::InvalidAccountsFile { .. })
        ));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{");
    }

    #[test]
    fn generated_names_take_smallest_free_suffix_across_chains() {
        let directory = tempdir().unwrap();
        let path = accounts_path(&directory);
        let repository = AccountRepository::try_new(&path).unwrap();
        let sepolia: ChainId = "SN_SEPOLIA".parse().unwrap();
        let mainnet: ChainId = "SN_MAIN".parse().unwrap();

        // The first insertion creates the file.
        let first = repository
            .insert_with_generated_name(sepolia, record(1))
            .unwrap()
            .value;
        assert_eq!(first.as_str(), "account-1");

        repository
            .insert(mainnet, AccountName::new("account-3").unwrap(), record(3))
            .unwrap();
        repository
            .insert(mainnet, AccountName::new("account-x").unwrap(), record(4))
            .unwrap();

        // `account-1` is taken on another chain, so it is not reused on mainnet.
        let second = repository
            .insert_with_generated_name(mainnet, record(2))
            .unwrap()
            .value;
        assert_eq!(second.as_str(), "account-2");
        let third = repository
            .insert_with_generated_name(sepolia, record(5))
            .unwrap()
            .value;
        assert_eq!(third.as_str(), "account-4");

        assert_eq!(
            repository.find(&mainnet, &second).unwrap().address,
            Felt::TWO
        );
        assert_eq!(
            repository.find(&sepolia, &third).unwrap().address,
            Felt::from(5)
        );

        let duplicate = repository.insert(sepolia, first.clone(), record(6));
        assert!(matches!(
            duplicate,
            Err(AccountsError::DuplicateAccount { account, .. }) if account == first
        ));
        assert_eq!(
            repository.find(&sepolia, &first).unwrap().address,
            Felt::ONE
        );
    }

    #[test]
    fn concurrent_generated_insertions_keep_every_record() {
        const WRITERS: u64 = 8;

        let directory = tempdir().unwrap();
        let path = accounts_path(&directory);
        let barrier = std::sync::Barrier::new(usize::try_from(WRITERS).unwrap());

        let mut names: Vec<String> = std::thread::scope(|scope| {
            let writers: Vec<_> = (1..=WRITERS)
                .map(|address| {
                    let (path, barrier) = (&path, &barrier);
                    scope.spawn(move || {
                        let repository = AccountRepository::try_new(path).unwrap();
                        let chain_id: ChainId = if address % 2 == 0 {
                            "SN_SEPOLIA"
                        } else {
                            "SN_MAIN"
                        }
                        .parse()
                        .unwrap();
                        barrier.wait();
                        let name = repository
                            .insert_with_generated_name(chain_id, record(address))
                            .unwrap()
                            .value;
                        let stored = repository.find(&chain_id, &name).unwrap();
                        assert_eq!(stored.address, Felt::from(address));
                        name.as_str().to_owned()
                    })
                })
                .collect();
            writers
                .into_iter()
                .map(|writer| writer.join().unwrap())
                .collect()
        });

        names.sort();
        let mut expected: Vec<String> = (1..=WRITERS).map(|id| format!("account-{id}")).collect();
        expected.sort();
        assert_eq!(names, expected);

        let registry = AccountRepository::try_new(&path)
            .unwrap()
            .load()
            .unwrap()
            .registry;
        let stored = registry
            .networks()
            .values()
            .map(BTreeMap::len)
            .sum::<usize>();
        assert_eq!(stored, usize::try_from(WRITERS).unwrap());
    }

    #[test]
    fn generated_insertion_migrates_with_backup_of_original_bytes() {
        let directory = tempdir().unwrap();
        let path = accounts_path(&directory);
        std::fs::write(&path, "{}").unwrap();
        let repository = AccountRepository::try_new(&path).unwrap();

        let result = repository
            .insert_with_generated_name("SN_SEPOLIA".parse().unwrap(), record(1))
            .unwrap();

        let backup_path = repository.backup_path(SourceVersion::V1);
        assert_eq!(
            result.migration_outcome,
            MigrationOutcome::Performed {
                from: SourceVersion::V1,
                to: SourceVersion::LATEST,
                backup_path: backup_path.clone(),
            }
        );
        assert_eq!(std::fs::read_to_string(&backup_path).unwrap(), "{}");
        assert!(repository.load().unwrap().source_version.is_latest());
    }

    #[test]
    fn migration_overwrites_existing_backup_with_owner_only_permissions() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let directory = tempdir().unwrap();
        let path = Utf8PathBuf::try_from(directory.path().join("accounts.json")).unwrap();
        fs::write(&path, "{}").unwrap();
        let repository = AccountRepository::try_new(&path).unwrap();
        let backup_path = repository.backup_path(SourceVersion::V1);
        fs::write(&backup_path, "stale").unwrap();
        fs::set_permissions(&backup_path, fs::Permissions::from_mode(0o644)).unwrap();

        repository.update_to_latest_schema().unwrap();

        assert_eq!(fs::read_to_string(&backup_path).unwrap(), "{}");
        let mode = fs::metadata(&backup_path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
