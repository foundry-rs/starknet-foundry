pub mod runner;
pub mod running_tests;

use std::path::PathBuf;

use anyhow::Result;
use assert_fs::fixture::PathCopy;
use project_root::get_project_root;
use scarb_api::ScarbCommand;
use semver::Version;

const DEFAULT_ASSERT_MACROS: Version = Version::new(2, 8, 5);
const MINIMAL_SCARB_FOR_CORRESPONDING_ASSERT_MACROS: Version = Version::new(2, 8, 5);

pub fn get_local_snforge_std_absolute_path() -> Result<PathBuf> {
    Ok(get_project_root()?.canonicalize()?.join("snforge_std"))
}

pub fn tempdir_with_tool_versions() -> Result<assert_fs::TempDir> {
    let project_root = get_project_root()?;
    let temp_dir = assert_fs::TempDir::new()?;
    temp_dir.copy_from(project_root, &[".tool-versions"])?;
    Ok(temp_dir)
}

pub fn get_assert_macros_version() -> Result<Version> {
    let scarb_version_output = ScarbCommand::version().run()?;
    let assert_macros_version =
        if scarb_version_output.scarb < MINIMAL_SCARB_FOR_CORRESPONDING_ASSERT_MACROS {
            DEFAULT_ASSERT_MACROS
        } else {
            scarb_version_output.cairo
        };
    Ok(assert_macros_version)
}
