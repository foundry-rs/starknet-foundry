use std::fs;
use std::io::{self, ErrorKind};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub(crate) fn ensure_regular_file(path: &impl AsRef<Path>, must_exist: bool) -> io::Result<()> {
    if !path.as_ref().exists() && !must_exist {
        return Ok(());
    }

    let metadata = std::fs::metadata(path)?;

    if metadata.is_symlink() {
        return Err(io::Error::new(
            ErrorKind::PermissionDenied,
            "refusing to access a file through a symlink",
        ));
    }

    if metadata.is_dir() {
        return Err(io::Error::new(
            ErrorKind::IsADirectory,
            format!("{} is a directory", path.as_ref().display()),
        ));
    }

    Ok(())
}

pub(crate) fn set_owner_only_permissions(path: impl AsRef<Path>) -> io::Result<()> {
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}
