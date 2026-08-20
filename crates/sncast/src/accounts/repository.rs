use std::fs::{self, File, OpenOptions};
use std::io::Write;

use camino::{Utf8Path, Utf8PathBuf};
use fs2::FileExt;
use tempfile::Builder;

use crate::accounts::schema::{DecodedAccountRegistry, SourceVersion};
use crate::accounts::{AccountName, AccountRecord, AccountRegistry, AccountsError, NetworkName};
use crate::helpers::filesystem::{reject_symlink, set_secret_permissions};

#[derive(Debug)]
pub struct MutationResult<T> {
    pub value: T,
    pub migrated_from_v1: bool,
}

#[derive(Clone, Debug)]
pub struct AccountRepository {
    path: Utf8PathBuf,
}

impl AccountRepository {
    #[must_use]
    pub fn new(path: Utf8PathBuf) -> Self {
        Self { path }
    }

    #[must_use]
    pub fn path(&self) -> &Utf8Path {
        &self.path
    }

    #[must_use]
    pub fn v1_backup_path(&self) -> Utf8PathBuf {
        v1_backup_path(&self.path)
    }

    pub fn file_exists(&self) -> anyhow::Result<()> {
        if !self.exists()? {
            anyhow::bail!(
                "Accounts file = {} does not exist! If you do not have an account create one with `account create` command or if you're using a custom accounts file, make sure to supply correct path to it with `--accounts-file` argument.",
                self.path
            );
        }
        Ok(())
    }

    pub fn load(&self) -> Result<DecodedAccountRegistry, AccountsError> {
        if !self.exists()? {
            return Err(AccountsError::FileNotFound {
                path: self.path.clone(),
            });
        }
        DecodedAccountRegistry::decode(&self.read()?)
            .map_err(|error| attach_file_path(error, &self.path))
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
        self.mutate(move |registry| {
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
        self.mutate(|registry| {
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
        operation: impl FnOnce(&mut AccountRegistry) -> Result<T, AccountsError>,
    ) -> Result<MutationResult<T>, AccountsError> {
        self.with_exclusive_lock(|| {
            let existed = self.exists()?;
            let original = if existed { self.read()? } else { Vec::new() };
            let mut decoded = DecodedAccountRegistry::decode(&original)
                .map_err(|error| attach_file_path(error, &self.path))?;
            let value = operation(&mut decoded.registry)?;
            let encoded = DecodedAccountRegistry::encode_v2(&decoded.registry)?;
            let migrated_from_v1 = existed && decoded.source_version == SourceVersion::V1;

            if migrated_from_v1 {
                self.write_backup_if_absent(&self.v1_backup_path(), &original)?;
            }
            self.write_atomic(&encoded)?;

            Ok(MutationResult {
                value,
                migrated_from_v1,
            })
        })
    }

    fn exists(&self) -> Result<bool, AccountsError> {
        reject_symlink(&self.path).map_err(|source| AccountsError::Storage {
            operation: "inspect accounts path",
            path: self.path.clone(),
            source,
        })?;
        Ok(self.path.exists())
    }

    fn read(&self) -> Result<Vec<u8>, AccountsError> {
        reject_symlink(&self.path).map_err(|source| AccountsError::Storage {
            operation: "inspect accounts path",
            path: self.path.clone(),
            source,
        })?;
        fs::read(&self.path).map_err(|source| AccountsError::Storage {
            operation: "read accounts file",
            path: self.path.clone(),
            source,
        })
    }

    fn write_atomic(&self, contents: &[u8]) -> Result<(), AccountsError> {
        reject_symlink(&self.path).map_err(|source| AccountsError::Storage {
            operation: "inspect accounts path",
            path: self.path.clone(),
            source,
        })?;
        let parent = ensure_parent(&self.path)?;
        let mut temporary = Builder::new()
            .prefix(".sncast-accounts-")
            .tempfile_in(parent)
            .map_err(|source| AccountsError::Storage {
                operation: "create temporary accounts file",
                path: self.path.clone(),
                source,
            })?;

        set_secret_permissions(temporary.path()).map_err(|source| AccountsError::Storage {
            operation: "set accounts file permissions",
            path: self.path.clone(),
            source,
        })?;
        temporary
            .write_all(contents)
            .map_err(|source| AccountsError::Storage {
                operation: "write temporary accounts file",
                path: self.path.clone(),
                source,
            })?;
        temporary.flush().map_err(|source| AccountsError::Storage {
            operation: "flush temporary accounts file",
            path: self.path.clone(),
            source,
        })?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|source| AccountsError::Storage {
                operation: "sync temporary accounts file",
                path: self.path.clone(),
                source,
            })?;

        temporary
            .persist(&self.path)
            .map_err(|error| AccountsError::Storage {
                operation: "replace accounts file",
                path: self.path.clone(),
                source: error.error,
            })?;
        sync_parent(parent, &self.path)
    }

    fn write_backup_if_absent(
        &self,
        path: &Utf8Path,
        contents: &[u8],
    ) -> Result<(), AccountsError> {
        reject_symlink(path).map_err(|source| AccountsError::Storage {
            operation: "inspect accounts path",
            path: path.to_owned(),
            source,
        })?;
        let parent = ensure_parent(path)?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        set_secret_open_options(&mut options);

        let mut file = match options.open(path) {
            Ok(file) => Ok(file),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Ok(()),
            Err(source) => Err(source),
        }
        .map_err(|source| AccountsError::Storage {
            operation: "create V1 backup",
            path: path.to_owned(),
            source,
        })?;
        file.write_all(contents)
            .map_err(|source| AccountsError::Storage {
                operation: "write V1 backup",
                path: path.to_owned(),
                source,
            })?;
        file.sync_all().map_err(|source| AccountsError::Storage {
            operation: "sync V1 backup",
            path: path.to_owned(),
            source,
        })?;
        sync_parent(parent, path)
    }

    fn with_exclusive_lock<T>(
        &self,
        operation: impl FnOnce() -> Result<T, AccountsError>,
    ) -> Result<T, AccountsError> {
        let parent = ensure_parent(&self.path)?;
        let lock_path = lock_path(&self.path);
        reject_symlink(&lock_path).map_err(|source| AccountsError::Storage {
            operation: "inspect accounts path",
            path: lock_path.clone(),
            source,
        })?;

        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);
        set_secret_open_options(&mut options);
        let lock = options
            .open(&lock_path)
            .map_err(|source| AccountsError::Storage {
                operation: "open accounts lock",
                path: lock_path,
                source,
            })?;
        lock.lock_exclusive()
            .map_err(|source| AccountsError::Storage {
                operation: "lock accounts file",
                path: self.path.clone(),
                source,
            })?;

        let result = operation();
        FileExt::unlock(&lock).map_err(|source| AccountsError::Storage {
            operation: "unlock accounts file",
            path: self.path.clone(),
            source,
        })?;
        sync_parent(parent, &self.path)?;
        result
    }
}

fn attach_file_path(error: AccountsError, file: &Utf8Path) -> AccountsError {
    match error {
        AccountsError::Schema { path, message } => AccountsError::SchemaFile {
            file: file.to_owned(),
            field: path,
            message,
        },
        error => error,
    }
}

fn v1_backup_path(path: &Utf8Path) -> Utf8PathBuf {
    let file_name = path.file_name().unwrap_or("accounts.json");
    path.with_file_name(format!("{file_name}.v1.bak"))
}

fn ensure_parent(path: &Utf8Path) -> Result<&Utf8Path, AccountsError> {
    let parent = path.parent().unwrap_or(Utf8Path::new("."));
    fs::create_dir_all(parent).map_err(|source| AccountsError::Storage {
        operation: "create accounts directory",
        path: parent.to_owned(),
        source,
    })?;
    Ok(parent)
}

fn lock_path(path: &Utf8Path) -> Utf8PathBuf {
    let file_name = path.file_name().unwrap_or("accounts.json");
    path.with_file_name(format!("{file_name}.lock"))
}

#[cfg(unix)]
fn set_secret_open_options(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o600);
}

#[cfg(not(unix))]
fn set_secret_open_options(_options: &mut OpenOptions) {}

#[cfg(unix)]
fn sync_parent(parent: &Utf8Path, accounts_path: &Utf8Path) -> Result<(), AccountsError> {
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|source| AccountsError::Storage {
            operation: "sync accounts directory",
            path: accounts_path.to_owned(),
            source,
        })
}

#[cfg(not(unix))]
fn sync_parent(_parent: &Utf8Path, _accounts_path: &Utf8Path) -> Result<(), AccountsError> {
    Ok(())
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

        let repository = AccountRepository::new(path.clone());
        let decoded = repository.load().unwrap();

        assert_eq!(decoded.source_version, SourceVersion::V1);
        assert_eq!(fs::read_to_string(&path).unwrap(), V1_ACCOUNT);
        assert!(!repository.v1_backup_path().exists());
    }

    #[test]
    fn file_exists_reports_missing_file() {
        let (_directory, path) = path();

        let error = AccountRepository::new(path.clone())
            .file_exists()
            .unwrap_err();

        assert!(error.to_string().contains(&path.to_string()));
    }

    #[test]
    fn successful_v1_mutation_writes_v2_and_backup() {
        let (_directory, path) = path();
        fs::write(&path, V1_ACCOUNT).unwrap();

        let repository = AccountRepository::new(path.clone());
        let result = repository
            .mutate(|registry| {
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
            fs::read_to_string(repository.v1_backup_path()).unwrap(),
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

        let repository = AccountRepository::new(path.clone());
        let result = repository.mutate(|_| {
            Err::<(), _>(AccountsError::MissingField {
                field: "address",
                operation: "test",
            })
        });

        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), V1_ACCOUNT);
        assert!(!repository.v1_backup_path().exists());
    }

    #[test]
    fn schema_errors_include_file_and_field_paths() {
        let (_directory, path) = path();
        fs::write(
            &path,
            r#"{"alpha-sepolia":{"alice":{"public_key":"not-a-felt","private_key":"0x1"}}}"#,
        )
        .unwrap();

        let error = AccountRepository::new(path.clone()).load().unwrap_err();

        assert!(matches!(
            error,
            AccountsError::SchemaFile { file, field, .. }
                if file == path && field.contains("alpha-sepolia.alice.public_key")
        ));
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

        AccountRepository::new(path.clone())
            .insert(
                NetworkName::new("alpha-sepolia").unwrap(),
                AccountName::new("alice").unwrap(),
                account,
            )
            .unwrap();

        assert_eq!(
            AccountRepository::new(path.clone())
                .load()
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
                    AccountRepository::new(path)
                        .insert(
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

        let decoded = AccountRepository::new(path.clone()).load().unwrap();
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
            .write_atomic(b"{\"version\":2}")
            .unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"{\"version\":2}");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn rejects_accounts_file_symlinks() {
        use std::os::unix::fs::symlink;

        let (directory, path) = path();
        let target = directory.path().join("target.json");
        fs::write(&target, "{}").unwrap();
        symlink(&target, &path).unwrap();

        assert!(matches!(
            AccountRepository::new(path).read(),
            Err(AccountsError::Storage {
                operation: "inspect accounts path",
                source,
                ..
            }) if source.kind() == std::io::ErrorKind::PermissionDenied
        ));
    }
}
