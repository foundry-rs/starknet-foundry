use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

pub(crate) fn reject_symlink(path: impl AsRef<Path>) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(io::Error::new(
            ErrorKind::PermissionDenied,
            "refusing to access a file through a symlink",
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
pub(crate) fn set_secret_permissions(path: impl AsRef<Path>) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
pub(crate) fn set_secret_permissions(_path: impl AsRef<Path>) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn accepts_missing_and_regular_paths() {
        let directory = tempdir().unwrap();
        let missing = directory.path().join("missing");
        let regular = directory.path().join("regular");
        fs::write(&regular, "content").unwrap();

        reject_symlink(missing).unwrap();
        reject_symlink(regular).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let directory = tempdir().unwrap();
        let target = directory.path().join("target");
        let link = directory.path().join("link");
        fs::write(&target, "content").unwrap();
        symlink(target, &link).unwrap();

        let error = reject_symlink(link).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::PermissionDenied);
    }

    #[cfg(unix)]
    #[test]
    fn sets_secret_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempdir().unwrap();
        let path = directory.path().join("secret");
        fs::write(&path, "content").unwrap();

        set_secret_permissions(&path).unwrap();

        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
