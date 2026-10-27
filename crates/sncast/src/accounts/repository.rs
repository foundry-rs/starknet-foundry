use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;

use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;

use crate::accounts::error::{FileType, StorageOperation};
use crate::accounts::schema::{DecodedAccountRegistry, SourceVersion};
use crate::accounts::{AccountName, AccountRecord, AccountRegistry, AccountsError, NetworkName};
use crate::helpers::filesystem::{LockableFile, NonMaterialized};

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

    pub fn update_record(
        &self,
        network: &str,
        name: &str,
        operation: impl FnOnce(&mut AccountRecord) -> Result<(), AccountsError>,
    ) -> Result<MutationResult<()>, AccountsError> {
        self.update(|registry| {
            registry
                .networks_mut()
                .get_mut(network)
                .and_then(|accounts| accounts.get_mut(name))
                .ok_or_else(|| AccountsError::AccountNotFound {
                    network: network.to_owned(),
                    account: name.to_owned(),
                })
                .and_then(operation)
        })
    }

    pub fn update_to_latest_schema(&self) -> Result<MigrationOutcome, AccountsError> {
        self.update(|_| Ok(()))
            .map(|result| result.migration_outcome)
    }

    pub fn generate_account_name(&self) -> Result<String, AccountsError> {
        let mut id = 1;

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

    pub fn load(&self) -> Result<DecodedAccountRegistry, AccountsError> {
        let exists = self.accounts_file.exists().map_to_accounts_error(
            StorageOperation::Inspect,
            FileType::AccountsFile,
            self.accounts_file.path(),
        )?;

        if !exists {
            return DecodedAccountRegistry::parse_json(&[]);
        }

        let contents = self
            .accounts_file
            .materialize()
            .map_to_accounts_error(
                StorageOperation::Inspect,
                FileType::AccountsFile,
                self.accounts_file.path(),
            )?
            .lock()
            .map_to_accounts_error(
                StorageOperation::Lock,
                FileType::AccountsFile,
                self.accounts_file.path(),
            )?
            .read()
            .map_to_accounts_error(
                StorageOperation::Read,
                FileType::AccountsFile,
                self.accounts_file.path(),
            )?;

        DecodedAccountRegistry::parse_json(&contents)
    }

    fn update<T>(
        &self,
        operation: impl FnOnce(&mut AccountRegistry) -> Result<T, AccountsError>,
    ) -> Result<MutationResult<T>, AccountsError> {
        let existed = self.accounts_file.exists().map_to_accounts_error(
            StorageOperation::Inspect,
            FileType::AccountsFile,
            self.accounts_file.path(),
        )?;

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
        } = DecodedAccountRegistry::parse_json(&original_contents)?;

        let mutation_outcome = operation(&mut registry)?;

        let needs_backup = !source_version.is_latest() && (existed || original_contents.is_empty());

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

        locked_file
            .write_atomic(&registry.encode()?)
            .map_to_accounts_error(
                StorageOperation::Write,
                FileType::AccountsFile,
                self.accounts_file.path(),
            )?;

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

        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true).mode(0o600);

        let mut file = options.open(&path).map_to_accounts_error(
            StorageOperation::Create,
            FileType::Backup,
            &path,
        )?;

        file.write_all(contents).map_to_accounts_error(
            StorageOperation::Write,
            FileType::Backup,
            &path,
        )?;

        Ok(path)
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
