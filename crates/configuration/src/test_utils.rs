use crate::CONFIG_FILENAME;
use std::fs;
use std::path::Path;
use tempfile::{TempDir, tempdir};

pub fn copy_config_to_dir(dest: impl AsRef<Path>, src_path: &str, additional_path: Option<&str>) {
    let dest = dest.as_ref();
    if let Some(dir) = additional_path {
        fs::create_dir_all(dest.join(dir)).expect("Failed to create directories in temp dir");
    }
    fs::copy(src_path, dest.join(CONFIG_FILENAME)).expect("Failed to copy config file to temp dir");
}

#[must_use]
pub fn copy_config_to_tempdir(src_path: &str, additional_path: Option<&str>) -> TempDir {
    let temp_dir = tempdir().expect("Failed to create a temporary directory");
    copy_config_to_dir(temp_dir.path(), src_path, additional_path);
    temp_dir
}
