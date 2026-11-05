use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::marker::PhantomData;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

use camino::{FromPathError, Utf8Path, Utf8PathBuf};

/// A file location that passed basic checks but may not exist yet.
pub struct NonMaterialized;

/// An existing regular file with a canonical path.
pub struct Materialized;

/// A materialized file whose sidecar lock is held until drop.
pub struct Locked;

/// A data file guarded by a sidecar lockfile.
///
/// Every write to the data file must go through [`LockableFile::write_atomic`] while holding the
/// lock. It replaces the file atomically (temporary file, fsync, rename), so readers that don't
/// take the lock ([`LockableFile::read_if_exists`]) always see a complete version of the file.
/// The lock only serializes read-modify-write cycles between writers.
pub struct LockableFile<State> {
    path: Utf8PathBuf,
    /// An open sidecar lockfile, separate from the data file which can be replaced during the lock.
    /// Only the `Locked` state holds this handle; dropping it releases the lock.
    lock: Option<File>,
    _state: PhantomData<State>,
}

impl<State> LockableFile<State> {
    #[must_use]
    pub fn path(&self) -> &Utf8Path {
        &self.path
    }
}

impl LockableFile<NonMaterialized> {
    pub fn try_new(path: &Utf8Path) -> io::Result<Self> {
        ensure_regular_file(path)?;

        Ok(Self {
            path: path.to_path_buf(),
            lock: None,
            _state: PhantomData,
        })
    }

    /// Reads the file without locking it, returning `None` if it doesn't exist.
    ///
    /// Doesn't create the file, its parent directories or the lockfile. This is sound only because
    /// all writes go through [`LockableFile::write_atomic`], so the file is never partially written.
    pub fn read_if_exists(&self) -> io::Result<Option<Vec<u8>>> {
        match fs::read(&self.path) {
            Ok(contents) => Ok(Some(contents)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn materialize(&self) -> io::Result<LockableFile<Materialized>> {
        if let Some(parent) = self
            .path
            .parent()
            .filter(|parent| !parent.as_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }

        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&self.path)
        {
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }

        let path = self.path.canonicalize_utf8()?;

        if !fs::metadata(&path)?.is_file() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("{} is not a regular file", path.as_str()),
            ));
        }

        if path.parent().is_none() {
            return Err(io::Error::new(
                ErrorKind::NotADirectory,
                format!("parent of a file at {} is not a directory", path.as_str()),
            ));
        }

        Ok(LockableFile {
            path,
            lock: None,
            _state: PhantomData,
        })
    }
}

impl LockableFile<Materialized> {
    pub fn lock(self) -> io::Result<LockableFile<Locked>> {
        let lockfile_path = Utf8PathBuf::from(format!("{}.lock", self.path));
        let lockfile = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(lockfile_path)?;
        lockfile.lock()?;

        Ok(LockableFile {
            path: self.path,
            lock: Some(lockfile),
            _state: PhantomData,
        })
    }
}

impl LockableFile<Locked> {
    pub fn read(&self) -> io::Result<Vec<u8>> {
        fs::read(&self.path)
    }

    /// Atomically replaces the file contents. This is the only supported way to write the file.
    pub fn write_atomic(&self, contents: &[u8]) -> io::Result<()> {
        debug_assert!(self.lock.is_some());
        write_atomic(&self.path, contents)
    }
}

/// Atomically replaces the file at `path` with `contents`, readable only by the owner.
///
/// Writes a temporary file in the same directory, fsyncs it, renames it over `path` and fsyncs
/// the directory. A crash leaves either the old or the new contents, never a partial file, and
/// permissions of an already existing file are not inherited.
///
/// Doesn't lock anything: writes to a [`LockableFile`] must go through [`LockableFile::write_atomic`].
pub fn write_atomic(path: &Utf8Path, contents: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_str().is_empty())
        .unwrap_or(Utf8Path::new("."));

    let mut temporary = tempfile::Builder::new()
        .prefix(".sncast-tmp-")
        .tempfile_in(parent)?;

    let temporary_path = temporary
        .path()
        .try_into()
        .map_err(FromPathError::into_io_error)?;

    set_owner_only_permissions(temporary_path)?;

    temporary.write_all(contents)?;
    temporary.flush()?;
    temporary.as_file().sync_all()?;
    temporary.persist(path)?;

    File::open(parent)?.sync_all()
}

pub fn set_owner_only_permissions(path: &Utf8Path) -> io::Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

/// Checks if the file at `path` is a regular file or can possibly represent a regular file if it doesn't exist.
pub fn ensure_regular_file(path: &Utf8Path) -> io::Result<()> {
    if path.file_name().is_none() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "file path must have a filename",
        ));
    }

    match fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => Err(io::Error::new(
            ErrorKind::IsADirectory,
            format!("{path} is a directory"),
        )),
        Ok(metadata) if !metadata.is_file() => Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("{path} is not a regular file"),
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}
