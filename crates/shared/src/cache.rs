use anyhow::{Context, Result};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub const CACHEDIR_TAG_FILENAME: &str = "CACHEDIR.TAG";

const CACHEDIR_TAG_CONTENTS: &str = "\
Signature: 8a477f597d28d172789f06886806bc55
# This file is a cache directory tag created by Starknet Foundry.
# For information about cache directory tags, see:
# https://bford.info/cachedir/
";

pub fn prepare_cache_dir(cache_dir: impl AsRef<Path>) -> Result<()> {
    let cache_dir = cache_dir.as_ref();
    fs::create_dir_all(cache_dir)
        .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;

    let tag_path = cache_dir.join(CACHEDIR_TAG_FILENAME);

    match fs::symlink_metadata(&tag_path) {
        Ok(_) => return Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => {
            return Err(err).with_context(|| {
                format!(
                    "Failed to read cache directory tag metadata: {}",
                    tag_path.display()
                )
            });
        }
    }

    fs::write(&tag_path, CACHEDIR_TAG_CONTENTS).with_context(|| {
        format!(
            "Failed to write cache directory tag: {}",
            tag_path.display()
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CACHEDIR_TAG_CONTENTS, CACHEDIR_TAG_FILENAME, prepare_cache_dir};
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
