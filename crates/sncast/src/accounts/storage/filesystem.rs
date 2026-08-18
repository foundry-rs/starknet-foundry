use std::fs::{self, File, OpenOptions};
use std::io::Write;

use camino::{Utf8Path, Utf8PathBuf};
use fs2::FileExt;
use tempfile::Builder;

use crate::accounts::AccountsError;
use crate::accounts::storage::AccountsStorage;

#[derive(Clone, Copy, Debug, Default)]
pub struct FileSystemAccountsStorage;

impl AccountsStorage for FileSystemAccountsStorage {
    fn exists(&self, path: &Utf8Path) -> Result<bool, AccountsError> {
        reject_symlink(path)?;
        Ok(path.exists())
    }

    fn read(&self, path: &Utf8Path) -> Result<Vec<u8>, AccountsError> {
        reject_symlink(path)?;
        fs::read(path).map_err(|source| storage_error("read accounts file", path, source))
    }

    fn write_atomic(&self, path: &Utf8Path, contents: &[u8]) -> Result<(), AccountsError> {
        reject_symlink(path)?;
        let parent = ensure_parent(path)?;
        let mut temporary = Builder::new()
            .prefix(".sncast-accounts-")
            .tempfile_in(parent)
            .map_err(|source| storage_error("create temporary accounts file", path, source))?;

        set_secret_permissions(temporary.path(), path)?;
        temporary
            .write_all(contents)
            .map_err(|source| storage_error("write temporary accounts file", path, source))?;
        temporary
            .flush()
            .map_err(|source| storage_error("flush temporary accounts file", path, source))?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|source| storage_error("sync temporary accounts file", path, source))?;

        temporary
            .persist(path)
            .map_err(|error| storage_error("replace accounts file", path, error.error))?;
        sync_parent(parent, path)?;
        Ok(())
    }

    fn write_backup_if_absent(
        &self,
        path: &Utf8Path,
        contents: &[u8],
    ) -> Result<(), AccountsError> {
        reject_symlink(path)?;
        let parent = ensure_parent(path)?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        set_secret_open_options(&mut options);

        let mut file = match options.open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Ok(()),
            Err(source) => return Err(storage_error("create V1 backup", path, source)),
        };
        file.write_all(contents)
            .map_err(|source| storage_error("write V1 backup", path, source))?;
        file.sync_all()
            .map_err(|source| storage_error("sync V1 backup", path, source))?;
        sync_parent(parent, path)
    }

    fn with_exclusive_lock<T>(
        &self,
        path: &Utf8Path,
        operation: impl FnOnce() -> Result<T, AccountsError>,
    ) -> Result<T, AccountsError> {
        let parent = ensure_parent(path)?;
        let lock_path = lock_path(path);
        reject_symlink(&lock_path)?;

        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);
        set_secret_open_options(&mut options);
        let lock = options
            .open(&lock_path)
            .map_err(|source| storage_error("open accounts lock", &lock_path, source))?;
        lock.lock_exclusive()
            .map_err(|source| storage_error("lock accounts file", path, source))?;

        let result = operation();
        FileExt::unlock(&lock)
            .map_err(|source| storage_error("unlock accounts file", path, source))?;
        sync_parent(parent, path)?;
        result
    }
}

fn ensure_parent(path: &Utf8Path) -> Result<&Utf8Path, AccountsError> {
    let parent = path.parent().unwrap_or(Utf8Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|source| storage_error("create accounts directory", parent, source))?;
    Ok(parent)
}

fn lock_path(path: &Utf8Path) -> Utf8PathBuf {
    let file_name = path.file_name().unwrap_or("accounts.json");
    path.with_file_name(format!("{file_name}.lock"))
}

fn reject_symlink(path: &Utf8Path) -> Result<(), AccountsError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(AccountsError::Symlink {
            path: path.to_owned(),
        }),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(storage_error("inspect accounts path", path, source)),
    }
}

fn storage_error(
    operation: &'static str,
    path: &Utf8Path,
    source: std::io::Error,
) -> AccountsError {
    AccountsError::Storage {
        operation,
        path: path.to_owned(),
        source,
    }
}

#[cfg(unix)]
fn set_secret_permissions(
    temporary_path: &std::path::Path,
    accounts_path: &Utf8Path,
) -> Result<(), AccountsError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(temporary_path, fs::Permissions::from_mode(0o600))
        .map_err(|source| storage_error("set accounts file permissions", accounts_path, source))
}

#[cfg(not(unix))]
fn set_secret_permissions(
    _temporary_path: &std::path::Path,
    _accounts_path: &Utf8Path,
) -> Result<(), AccountsError> {
    Ok(())
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
        .map_err(|source| storage_error("sync accounts directory", accounts_path, source))
}

#[cfg(not(unix))]
fn sync_parent(_parent: &Utf8Path, _accounts_path: &Utf8Path) -> Result<(), AccountsError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn writes_complete_file_with_secret_permissions() {
        let directory = tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(directory.path().join("accounts.json")).unwrap();

        FileSystemAccountsStorage
            .write_atomic(&path, b"{\"version\":2}")
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

        let directory = tempdir().unwrap();
        let target = directory.path().join("target.json");
        let link = directory.path().join("accounts.json");
        fs::write(&target, "{}").unwrap();
        symlink(&target, &link).unwrap();
        let link = Utf8PathBuf::from_path_buf(link).unwrap();

        assert!(matches!(
            FileSystemAccountsStorage.read(&link),
            Err(AccountsError::Symlink { .. })
        ));
    }
}
