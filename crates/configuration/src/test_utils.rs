use crate::CONFIG_FILENAME;
use std::fs;
use tempfile::{TempDir, tempdir};

#[must_use]
pub fn copy_config_to_tempdir(src_path: &str, additional_path: Option<&str>) -> TempDir {
    let temp_dir = tempdir().expect("Failed to create a temporary directory");
    if let Some(dir) = additional_path {
        let path = temp_dir.path().join(dir);
        fs::create_dir_all(path).expect("Failed to create directories in temp dir");
    }
    let temp_dir_file_path = temp_dir.path().join(CONFIG_FILENAME);
    fs::copy(src_path, temp_dir_file_path).expect("Failed to copy config file to temp dir");

    temp_dir
}
