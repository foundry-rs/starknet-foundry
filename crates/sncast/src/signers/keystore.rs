use std::fs;

use camino::Utf8Path;
use starknet_rust::signers::SigningKey;
use starknet_types_core::felt::Felt;

use crate::helpers::filesystem::{reject_symlink, set_secret_permissions};
use crate::signers::{KeystoreError, SignerError};

/// Physical access to native encrypted signer files.
#[derive(Clone, Copy, Debug, Default)]
pub struct KeystoreFile;

impl KeystoreFile {
    pub fn decrypt(path: &Utf8Path, password: &str) -> Result<SigningKey, SignerError> {
        reject_symlink(path).map_err(|source| KeystoreError::Inspect {
            path: path.to_owned(),
            source,
        })?;
        SigningKey::from_keystore(path, password).map_err(|source| {
            KeystoreError::Decrypt {
                path: path.to_owned(),
                source,
            }
            .into()
        })
    }

    pub fn create(path: &Utf8Path, private_key: Felt, password: &str) -> Result<(), KeystoreError> {
        reject_symlink(path).map_err(|source| KeystoreError::Inspect {
            path: path.to_owned(),
            source,
        })?;
        if path.exists() {
            Err(KeystoreError::AlreadyExists {
                path: path.to_owned(),
            })?;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| KeystoreError::CreateDirectory {
                keystore_file_parent_path: parent.to_path_buf(),
                source,
            })?;
        }

        SigningKey::from_secret_scalar(private_key)
            .save_as_keystore(path, password)
            .map_err(|source| KeystoreError::Create {
                path: path.to_owned(),
                source,
            })?;
        set_secret_permissions(path).map_err(|source| KeystoreError::SetSecretPermissions {
            path: path.to_owned(),
            source,
        })?;
        Ok(())
    }

    pub fn remove(path: &Utf8Path) -> Result<(), KeystoreError> {
        reject_symlink(path).map_err(|source| KeystoreError::Inspect {
            path: path.to_owned(),
            source,
        })?;
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
        .map_err(|source| KeystoreError::Remove {
            path: path.to_owned(),
            source,
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
            Err(KeystoreError::AlreadyExists { .. })
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
            Err(SignerError::Keystore(KeystoreError::Inspect {
                path: error_path,
                ..
            })) if error_path == path
        ));
    }
}
