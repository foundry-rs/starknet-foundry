use std::{fs::File, io::Write};

use anyhow::Result;
use tempfile::TempDir;

use crate::helpers::fixtures::copy_directory_to_tempdir;
use configuration::CONFIG_FILENAME;

pub fn copy_directory_to_tempdir_with_config(
    src_dir: String,
    cast_config_content: String,
) -> Result<TempDir> {
    let temp_dir = copy_directory_to_tempdir(src_dir);

    let mut file = File::create(temp_dir.path().join(CONFIG_FILENAME))
        .expect("Unable to create a temporary accounts file");

    file.write_all(cast_config_content.as_bytes())
        .expect("Unable to write test data to a temporary file");

    file.flush().expect("Unable to flush a temporary file");

    Ok(temp_dir)
}
