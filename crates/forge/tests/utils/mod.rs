pub mod runner;
pub mod running_tests;

pub use crate::test_case;

use anyhow::Result;
use assert_fs::fixture::PathCopy;
use camino::Utf8PathBuf;
use project_root::get_project_root;
use scarb_api::version::scarb_version;
use semver::Version;
use std::str::FromStr;

pub fn tempdir_with_tool_versions() -> Result<assert_fs::TempDir> {
    let project_root = get_project_root()?;
    let temp_dir = assert_fs::TempDir::new()?;
    temp_dir.copy_from(project_root, &[".tool-versions"])?;
    Ok(temp_dir)
}

pub fn get_assert_macros_version() -> Result<Version> {
    Ok(scarb_version()?.cairo)
}

#[must_use]
pub fn get_std_name() -> String {
    "snforge_std".to_string()
}

pub fn get_std_path() -> Result<String> {
    let name = get_std_name();
    Ok(Utf8PathBuf::from_str(&format!("../../{name}"))?
        .canonicalize_utf8()?
        .to_string())
}

pub fn get_snforge_std_entry() -> Result<String> {
    let name = get_std_name();
    let path = get_std_path()?;

    Ok(format!("{name} = {{ path = \"{path}\" }}"))
}
