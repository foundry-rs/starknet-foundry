pub mod runner;
pub mod running_tests;

use std::path::PathBuf;

use anyhow::Result;
use assert_fs::fixture::PathCopy;
use project_root::get_project_root;

pub fn get_local_snforge_std_absolute_path() -> Result<PathBuf> {
    Ok(get_project_root()?.canonicalize()?.join("snforge_std"))
}

pub fn tempdir_with_tool_versions() -> Result<assert_fs::TempDir> {
    let project_root = get_project_root()?;
    let temp_dir = assert_fs::TempDir::new()?;
    temp_dir.copy_from(project_root, &[".tool-versions"])?;
    Ok(temp_dir)
}
