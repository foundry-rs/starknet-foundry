use super::constants::CONFIG_FILENAME;
use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use std::fs;

pub fn find_config_file_relative_to(current_dir: &Utf8PathBuf) -> Result<Utf8PathBuf> {
    current_dir
        .ancestors()
        .find(|path| fs::metadata(path.join(CONFIG_FILENAME)).is_ok())
        .map(|path| path.join(CONFIG_FILENAME))
        .ok_or_else(|| {
            anyhow!("Failed to find sncast.toml - not found in current nor any parent directories")
        })
}

pub fn find_config_file() -> Result<Utf8PathBuf> {
    find_config_file_relative_to(&Utf8PathBuf::try_from(
        std::env::current_dir().expect("Failed to get current directory"),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn copy_config_to_tempdir(src_path: &str, additional_path: Option<&str>) -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create a temporary directory");
        if let Some(dir) = additional_path {
            let path = temp_dir.path().join(dir);
            fs::create_dir_all(&path).expect("Failed to create directories in temp dir");
        };
        let temp_dir_file_path = temp_dir.path().join(CONFIG_FILENAME);
        fs::copy(src_path, temp_dir_file_path).expect("Failed to copy config file to temp dir");
        temp_dir
    }

    #[test]
    fn find_config_in_current_dir() {
        let tempdir = copy_config_to_tempdir("tests/data/files/correct_sncast.toml", None);
        let path = find_config_file_relative_to(
            &Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap(),
        )
        .unwrap();
        assert_eq!(path, tempdir.path().join(CONFIG_FILENAME));
    }

    #[test]
    fn find_config_in_parent_dir() {
        let tempdir =
            copy_config_to_tempdir("tests/data/files/correct_sncast.toml", Some("childdir"));
        let path = find_config_file_relative_to(
            &Utf8PathBuf::try_from(tempdir.path().to_path_buf().join("childdir")).unwrap(),
        )
        .unwrap();
        assert_eq!(path, tempdir.path().join(CONFIG_FILENAME));
    }

    #[test]
    fn find_config_in_parent_dir_two_levels() {
        let tempdir = copy_config_to_tempdir(
            "tests/data/files/correct_sncast.toml",
            Some("childdir1/childdir2"),
        );
        let path = find_config_file_relative_to(
            &Utf8PathBuf::try_from(tempdir.path().to_path_buf().join("childdir1/childdir2"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(path, tempdir.path().join(CONFIG_FILENAME));
    }

    #[test]
    fn find_config_in_parent_dir_available_in_multiple_parents() {
        let tempdir =
            copy_config_to_tempdir("tests/data/files/correct_sncast.toml", Some("childdir1"));
        fs::copy(
            "tests/data/files/correct_sncast.toml",
            tempdir.path().join("childdir1").join(CONFIG_FILENAME),
        )
        .expect("Failed to copy config file to temp dir");
        let path = find_config_file_relative_to(
            &Utf8PathBuf::try_from(tempdir.path().to_path_buf().join("childdir1")).unwrap(),
        )
        .unwrap();
        assert_eq!(path, tempdir.path().join("childdir1").join(CONFIG_FILENAME));
    }

    #[test]
    fn no_config_in_current_nor_parent_dir() {
        let tempdir = TempDir::new().expect("Failed to create a temporary directory");
        assert!(
            find_config_file_relative_to(
                &Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()
            )
            .is_err(),
            "Failed to find sncast.toml - not found in current nor any parent directories"
        );
    }
}
