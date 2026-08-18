//! Physical storage for accounts files.

mod filesystem;

use camino::Utf8Path;

use crate::accounts::AccountsError;

pub use filesystem::FileSystemAccountsStorage;

pub trait AccountsStorage {
    fn exists(&self, path: &Utf8Path) -> Result<bool, AccountsError>;
    fn read(&self, path: &Utf8Path) -> Result<Vec<u8>, AccountsError>;
    fn write_atomic(&self, path: &Utf8Path, contents: &[u8]) -> Result<(), AccountsError>;
    fn write_backup_if_absent(&self, path: &Utf8Path, contents: &[u8])
    -> Result<(), AccountsError>;

    fn with_exclusive_lock<T>(
        &self,
        path: &Utf8Path,
        operation: impl FnOnce() -> Result<T, AccountsError>,
    ) -> Result<T, AccountsError>;
}
