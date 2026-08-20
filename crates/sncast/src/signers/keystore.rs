use std::fs;

use camino::Utf8Path;
use starknet_rust::signers::SigningKey;
use starknet_types_core::felt::Felt;

use crate::helpers::filesystem::{reject_symlink, set_secret_permissions};
use crate::signers::SignerError;

/// Physical access to native encrypted signer files.
#[derive(Clone, Copy, Debug, Default)]
pub struct KeystoreFile;

impl KeystoreFile {
    pub fn decrypt(path: &Utf8Path, password: &str) -> Result<SigningKey, SignerError> {
        reject_symlink(path).map_err(|error| SignerError::KeystoreStorage {
            operation: "inspect",
            path: path.to_owned(),
            message: error.to_string(),
        })?;
        SigningKey::from_keystore(path, password).map_err(|error| SignerError::InvalidKeystore {
            path: path.to_owned(),
            message: error.to_string(),
        })
    }

    pub fn create(path: &Utf8Path, private_key: Felt, password: &str) -> Result<(), SignerError> {
        reject_symlink(path).map_err(|error| SignerError::KeystoreStorage {
            operation: "inspect",
            path: path.to_owned(),
            message: error.to_string(),
        })?;
        if path.exists() {
            return Err(SignerError::KeystoreAlreadyExists {
                path: path.to_owned(),
            });
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| SignerError::KeystoreStorage {
                operation: "create keystore directory",
                path: path.to_owned(),
                message: error.to_string(),
            })?;
        }

        SigningKey::from_secret_scalar(private_key)
            .save_as_keystore(path, password)
            .map_err(|error| SignerError::KeystoreStorage {
                operation: "create",
                path: path.to_owned(),
                message: error.to_string(),
            })?;
        set_secret_permissions(path).map_err(|error| SignerError::KeystoreStorage {
            operation: "set permissions on",
            path: path.to_owned(),
            message: error.to_string(),
        })?;
        Ok(())
    }

    pub fn remove(path: &Utf8Path) -> Result<(), SignerError> {
        reject_symlink(path).map_err(|error| SignerError::KeystoreStorage {
            operation: "inspect",
            path: path.to_owned(),
            message: error.to_string(),
        })?;
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
        .map_err(|error| SignerError::KeystoreStorage {
            operation: "remove",
            path: path.to_owned(),
            message: error.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use camino::Utf8PathBuf;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn creates_and_decrypts_keystore_with_secret_permissions() {
        let directory = tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(directory.path().join("keys/alice.json")).unwrap();

        KeystoreFile::create(&path, Felt::ONE, "secret").unwrap();
        let decrypted = KeystoreFile::decrypt(&path, "secret").unwrap();

        assert_eq!(decrypted.secret_scalar(), Felt::ONE);
        assert!(matches!(
            KeystoreFile::create(&path, Felt::TWO, "secret"),
            Err(SignerError::KeystoreAlreadyExists { .. })
        ));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn rejects_keystore_symlinks_as_storage_errors() {
        use std::os::unix::fs::symlink;

        let directory = tempdir().unwrap();
        let target = directory.path().join("target.json");
        let path = directory.path().join("keystore.json");
        fs::write(&target, "{}").unwrap();
        symlink(target, &path).unwrap();
        let path = Utf8PathBuf::from_path_buf(path).unwrap();

        assert!(matches!(
            KeystoreFile::decrypt(&path, "secret"),
            Err(SignerError::KeystoreStorage {
                operation: "inspect",
                path: error_path,
                ..
            }) if error_path == path
        ));
    }
}
