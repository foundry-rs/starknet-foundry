use std::collections::HashSet;
use std::collections::btree_map::Entry;
use std::io;

use camino::{Utf8Path, Utf8PathBuf};
use foundry_ui::components::warning::WarningMessage;
use serde::Serialize;

use crate::accounts::error::{FileType, StorageOperation};
use crate::accounts::schema::{DecodedAccountRegistry, SourceVersion};
use crate::accounts::{AccountName, AccountRecord, AccountRegistry, AccountsError, ChainId};
use crate::helpers::filesystem::{self, LockableFile, NonMaterialized};
use crate::response::ui::UI;

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

    pub fn insert(
        &self,
        chain_id: ChainId,
        name: AccountName,
        record: AccountRecord,
    ) -> Result<MutationResult<()>, AccountsError> {
        self.update(false, move |registry| {
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
        })
    }

    pub fn remove(
        &self,
        chain_id: &ChainId,
        name: &AccountName,
    ) -> Result<MutationResult<AccountRecord>, AccountsError> {
        self.update(false, |registry| {
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
        self.update(false, |registry| {
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
        self.update(true, |_| Ok(()))
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

    /// Reads the accounts file without locking it; see [`LockableFile::read_if_exists`].
    pub fn load(&self) -> Result<DecodedAccountRegistry, AccountsError> {
        let contents = self
            .accounts_file
            .read_if_exists()
            .map_to_accounts_error(
                StorageOperation::Read,
                FileType::AccountsFile,
                self.accounts_file.path(),
            )?
            .unwrap_or_default();

        self.decode(&contents)
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
        migration_only: bool,
        operation: impl FnOnce(&mut AccountRegistry) -> Result<T, AccountsError>,
    ) -> Result<MutationResult<T>, AccountsError> {
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

        if !migration_only || !source_version.is_latest() {
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
            ui.print_warning(WarningMessage::new(
                format!("Accounts file was migrated from schema version {from} to {to}; a backup of the original was saved at {backup_path}"),
            ));
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
