pub mod runner;
pub mod running_tests;

use anyhow::Result;
use assert_fs::fixture::PathCopy;
use camino::Utf8PathBuf;
use forge::MINIMAL_SCARB_VERSION_FOR_V2_MACROS_REQUIREMENT;
use project_root::get_project_root;
use scarb_api::ScarbCommand;
use semver::Version;
use std::str::FromStr;

const DEFAULT_ASSERT_MACROS: Version = Version::new(0, 1, 0);
const MINIMAL_SCARB_FOR_CORRESPONDING_ASSERT_MACROS: Version = Version::new(2, 8, 0);

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

#[must_use]
pub fn get_std_name() -> String {
    if use_snforge_std_deprecated() {
        "snforge_std_deprecated".to_string()
    } else {
        "snforge_std".to_string()
    }
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

#[must_use]
pub fn use_snforge_std_deprecated() -> bool {
    let scarb_version_output = ScarbCommand::version()
        .run()
        .expect("Failed to get scarb version");
    scarb_version_output.scarb < MINIMAL_SCARB_VERSION_FOR_V2_MACROS_REQUIREMENT
}
