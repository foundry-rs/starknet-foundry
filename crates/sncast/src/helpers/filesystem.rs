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
        if path.file_name().is_none() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "file path must have a filename",
            ));
        }
        match fs::metadata(path) {
            Ok(metadata) if metadata.is_dir() => {
                return Err(io::Error::new(
                    ErrorKind::IsADirectory,
                    format!("{path} is a directory"),
                ));
            }
            Ok(metadata) if !metadata.is_file() => {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("{path} is not a regular file"),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }

        Ok(Self {
            path: path.to_path_buf(),
            lock: None,
            _state: PhantomData,
        })
    }

    pub fn exists(&self) -> io::Result<bool> {
        self.path.try_exists()
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
    pub fn lock(&self) -> io::Result<LockableFile<Locked>> {
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
            path: self.path.clone(),
            lock: Some(lockfile),
            _state: PhantomData,
        })
    }
}

impl LockableFile<Locked> {
    pub fn read(&self) -> io::Result<Vec<u8>> {
        fs::read(&self.path)
    }

    pub fn write_atomic(&self, contents: &[u8]) -> io::Result<()> {
        let mut temporary = tempfile::Builder::new()
            .prefix(".sncast-accounts-")
            .tempfile_in(self.parent())?;

        let path = temporary
            .path()
            .try_into()
            .map_err(FromPathError::into_io_error)?;

        set_owner_only_permissions(path)?;

        temporary.write_all(contents)?;
        temporary.flush()?;
        temporary.as_file().sync_all()?;
        temporary.persist(&self.path).map_err(|error| error.error)?;

        self.sync_parent()?;

        Ok(())
    }

    fn parent(&self) -> &Utf8Path {
        debug_assert!(self.lock.is_some());
        self.path.parent().expect("parent was verified")
    }

    fn sync_parent(&self) -> io::Result<()> {
        File::open(self.parent()).and_then(|directory| directory.sync_all())
    }
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
