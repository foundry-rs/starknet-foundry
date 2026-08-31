use anyhow::{Context, Result, bail, ensure};
use camino::{Utf8Path, Utf8PathBuf};
use std::env::{self, VarError};
use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::Path;

pub const DEFAULT_CACHE_DIR: &str = ".snfoundry_cache";
pub const USC_CACHE_DIR: &str = "universal-sierra-compiler";

pub const CACHEDIR_TAG_FILENAME: &str = "CACHEDIR.TAG";
pub const CACHEDIR_TAG_CONTENTS: &str = "\
Signature: 8a477f597d28d172789f06886806bc55
# This file is a cache directory tag created by Starknet Foundry.
# For information about cache directory tags, see:
# https://bford.info/cachedir/
";

pub fn resolve_cache_dir(workspace_root: &Utf8Path) -> Result<Utf8PathBuf> {
    resolve_cache_dir_impl(workspace_root, env::var("SNFOUNDRY_CACHE"))
}

fn resolve_cache_dir_impl(
    workspace_root: &Utf8Path,
    cache_var: Result<String, VarError>,
) -> Result<Utf8PathBuf> {
    match cache_var {
        Ok(cache_dir) => {
            let cache_dir = Utf8PathBuf::from(cache_dir);
            ensure!(
                cache_dir.is_absolute(),
                "SNFOUNDRY_CACHE must be an absolute path"
            );
            Ok(cache_dir)
        }
        Err(VarError::NotPresent) => Ok(workspace_root.join(DEFAULT_CACHE_DIR)),
        Err(VarError::NotUnicode(_)) => {
            bail!("SNFOUNDRY_CACHE must be a valid UTF-8 string")
        }
    }
}

pub fn prepare_cache_dir(cache_dir: impl AsRef<Path>) -> Result<()> {
    let cache_dir = cache_dir.as_ref();
    fs::create_dir_all(cache_dir)
        .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;

    let tag_path = cache_dir.join(CACHEDIR_TAG_FILENAME);

    match File::create_new(&tag_path) {
        Ok(mut file) => file
            .write_all(CACHEDIR_TAG_CONTENTS.as_bytes())
            .with_context(|| {
                format!(
                    "Failed to write cache directory tag: {}",
                    tag_path.display()
                )
            }),
        Err(err) if err.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(err) => Err(err).with_context(|| {
            format!(
                "Failed to create cache directory tag: {}",
                tag_path.display()
            )
        }),
    }?;

    Ok(())
}

#[must_use]
pub fn usc_cache_dir(cache_dir: &Utf8PathBuf) -> Utf8PathBuf {
    cache_dir.join(USC_CACHE_DIR)
}

#[cfg(test)]
mod tests {
    use super::{
        CACHEDIR_TAG_CONTENTS, CACHEDIR_TAG_FILENAME, DEFAULT_CACHE_DIR, prepare_cache_dir,
        resolve_cache_dir_impl,
    };
    use camino::Utf8Path;
    use std::env::VarError;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn creates_cache_dir_with_tag() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join(".snfoundry_cache");

        prepare_cache_dir(&cache_dir).unwrap();

        let tag = fs::read_to_string(cache_dir.join(CACHEDIR_TAG_FILENAME)).unwrap();
        assert_eq!(tag, CACHEDIR_TAG_CONTENTS);
    }

    #[test]
    fn defaults_to_workspace_subdir_when_var_unset() {
        let workspace = Utf8Path::new("/tmp/workspace");
        assert_eq!(
            resolve_cache_dir_impl(workspace, Err(VarError::NotPresent)).unwrap(),
            workspace.join(DEFAULT_CACHE_DIR)
        );
    }

    #[test]
    fn accepts_absolute_custom_path() {
        let resolved = resolve_cache_dir_impl(
            Utf8Path::new("/tmp/workspace"),
            Ok("/var/cache/snfoundry".to_string()),
        )
        .unwrap();
        assert_eq!(resolved, Utf8Path::new("/var/cache/snfoundry"));
    }

    #[test]
    fn rejects_relative_custom_path() {
        let err = resolve_cache_dir_impl(
            Utf8Path::new("/tmp/workspace"),
            Ok("relative/cache".to_string()),
        )
        .unwrap_err();
        assert_eq!(err.to_string(), "SNFOUNDRY_CACHE must be an absolute path");
    }

    #[test]
    fn rejects_empty_string() {
        let err =
            resolve_cache_dir_impl(Utf8Path::new("/tmp/workspace"), Ok(String::new())).unwrap_err();
        assert_eq!(err.to_string(), "SNFOUNDRY_CACHE must be an absolute path");
    }

    #[test]
    fn creates_cache_dir_tag_when_missing() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join(".snfoundry_cache");
        fs::create_dir(&cache_dir).unwrap();

        prepare_cache_dir(&cache_dir).unwrap();

        let tag = fs::read_to_string(cache_dir.join(CACHEDIR_TAG_FILENAME)).unwrap();
        assert_eq!(tag, CACHEDIR_TAG_CONTENTS);
    }

    #[test]
    fn leaves_existing_cache_dir_tag_unchanged() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join(".snfoundry_cache");
        fs::create_dir(&cache_dir).unwrap();
        let tag_path = cache_dir.join(CACHEDIR_TAG_FILENAME);
        let existing_tag = "\
Signature: 8a477f597d28d172789f06886806bc55
# Existing cache tag.
";
        fs::write(&tag_path, existing_tag).unwrap();

        prepare_cache_dir(&cache_dir).unwrap();

        let tag = fs::read_to_string(tag_path).unwrap();
        assert_eq!(tag, existing_tag);
    }
}
