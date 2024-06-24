pub mod runner;
pub mod running_tests;

use anyhow::Result;
use assert_fs::fixture::PathCopy;

pub fn tempdir_with_tool_versions() -> Result<assert_fs::TempDir> {
    let project_root = project_root::get_project_root()?;
    let temp_dir = assert_fs::TempDir::new()?;
    temp_dir.copy_from(project_root, &[".tool-versions"])?;
    Ok(temp_dir)
}
