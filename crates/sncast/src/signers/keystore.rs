use std::fs;

use camino::Utf8Path;
use starknet_rust::signers::SigningKey;
use starknet_types_core::felt::Felt;

use crate::signers::SignerError;

/// Physical access to native encrypted signer files.
#[derive(Clone, Copy, Debug, Default)]
pub struct KeystoreFile;

impl KeystoreFile {
    pub fn decrypt(path: &Utf8Path, password: &str) -> Result<SigningKey, SignerError> {
        reject_symlink(path)?;
        SigningKey::from_keystore(path, password).map_err(|error| SignerError::InvalidKeystore {
            path: path.to_owned(),
            message: error.to_string(),
        })
    }

    pub fn create(path: &Utf8Path, private_key: Felt, password: &str) -> Result<(), SignerError> {
        reject_symlink(path)?;
        if path.exists() {
            return Err(SignerError::KeystoreAlreadyExists {
                path: path.to_owned(),
            });
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| storage_error("create keystore directory", path, error))?;
        }

        SigningKey::from_secret_scalar(private_key)
            .save_as_keystore(path, password)
            .map_err(|error| SignerError::KeystoreStorage {
                operation: "create",
                path: path.to_owned(),
                message: error.to_string(),
            })?;
        set_secret_permissions(path)?;
        Ok(())
    }

    pub fn remove(path: &Utf8Path) -> Result<(), SignerError> {
        reject_symlink(path)?;
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(storage_error("remove", path, error)),
        }
    }
}

fn reject_symlink(path: &Utf8Path) -> Result<(), SignerError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(SignerError::KeystoreStorage {
            operation: "access",
            path: path.to_owned(),
            message: "refusing to access a keystore through a symlink".to_owned(),
        }),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(storage_error("inspect", path, error)),
    }
}

fn storage_error(operation: &'static str, path: &Utf8Path, error: std::io::Error) -> SignerError {
    SignerError::KeystoreStorage {
        operation,
        path: path.to_owned(),
        message: error.to_string(),
    }
}

#[cfg(unix)]
fn set_secret_permissions(path: &Utf8Path) -> Result<(), SignerError> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|error| storage_error("set permissions on", path, error))
}

#[cfg(not(unix))]
fn set_secret_permissions(_path: &Utf8Path) -> Result<(), SignerError> {
    Ok(())
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
}
