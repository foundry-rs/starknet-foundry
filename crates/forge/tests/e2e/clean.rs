use super::common::runner::{runner, setup_package, test_runner};
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use scarb_api::metadata::MetadataCommandExt;
use scarb_api::ScarbCommand;
use shared::test_utils::output_assert::assert_stdout_contains;
use std::path::Path;

const COVERAGE_DIR: &str = "coverage";
const PROFILE_DIR: &str = "profile";
const CACHE_DIR: &str = ".snfoundry_cache";
const TRACE_DIR: &str = "snfoundry_trace";

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct CleanComponentsState {
    coverage: bool,
    profile: bool,
    cache: bool,
    trace: bool,
}

#[test]
#[cfg_attr(not(feature = "scarb_2_8_3"), ignore)]
fn test_clean_coverage() {
    let temp_dir = setup_package("coverage_project");

    let clean_components_state = CleanComponentsState {
        coverage: true,
        profile: false,
        cache: true,
        trace: true,
    };

    generate_clean_components(clean_components_state, &temp_dir);

    runner(&temp_dir)
        .arg("clean")
        .arg("coverage")
        .arg("trace")
        .assert()
        .success();

    let expected_state = CleanComponentsState {
        coverage: false,
        profile: false,
        cache: true,
        trace: false,
    };

    assert_eq!(
        check_clean_components_state(temp_dir.path()),
        expected_state
    );
}

#[test]
#[cfg(not(target_os = "windows"))]
fn test_clean_profile() {
    let temp_dir = setup_package("coverage_project");

    let clean_components_state = CleanComponentsState {
        coverage: false,
        profile: true,
        cache: true,
        trace: true,
    };

    generate_clean_components(clean_components_state, &temp_dir);

    runner(&temp_dir)
        .arg("clean")
        .arg("profile")
        .assert()
        .success();

    let expected_state = CleanComponentsState {
        coverage: false,
        profile: false,
        cache: true,
        trace: true,
    };

    assert_eq!(
        check_clean_components_state(temp_dir.path()),
        expected_state
    );
}

#[test]
fn test_clean_cache() {
    let temp_dir = setup_package("coverage_project");

    let clean_components_state = CleanComponentsState {
        coverage: false,
        profile: false,
        cache: true,
        trace: false,
    };

    generate_clean_components(clean_components_state, &temp_dir);

    runner(&temp_dir)
        .arg("clean")
        .arg("cache")
        .assert()
        .success();

    let expected_state = CleanComponentsState {
        coverage: false,
        profile: false,
        cache: false,
        trace: false,
    };

    assert_eq!(
        check_clean_components_state(temp_dir.path()),
        expected_state
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_2_8_3"), ignore)]
fn test_clean_all() {
    let temp_dir = setup_package("coverage_project");

    let clean_components_state = CleanComponentsState {
        coverage: true,
        cache: true,
        trace: true,
        profile: true,
    };

    generate_clean_components(clean_components_state, &temp_dir);

    runner(&temp_dir).arg("clean").arg("all").assert().success();

    assert_eq!(
        check_clean_components_state(temp_dir.path()),
        clean_components_state
    );
}

#[test]
fn test_clean_all_and_component() {
    let temp_dir = setup_package("coverage_project");

    let clean_components_state = CleanComponentsState {
        coverage: false,
        cache: true,
        trace: true,
        profile: false,
    };
    generate_clean_components(clean_components_state, &temp_dir);

    // This command should fail because 'all' cannot be combined with other components
    let output = runner(&temp_dir)
        .arg("clean")
        .arg("all")
        .arg("cache")
        .assert()
        .failure();

    assert_stdout_contains(
        output,
        "[ERROR] The 'all' component cannot be combined with other components",
    );
}

fn generate_clean_components(state: CleanComponentsState, temp_dir: &TempDir) {
    let args = match state {
        CleanComponentsState {
            coverage: true,
            trace: true,
            cache: true,
            profile: false,
        } => {
            vec!["--coverage"]
        }
        CleanComponentsState {
            profile: true,
            trace: true,
            cache: true,
            coverage: false,
        } => {
            vec!["--build-profile"]
        }
        CleanComponentsState {
            trace: true,
            cache: true,
            profile: false,
            coverage: false,
        } => {
            vec!["--save-trace-data"]
        }
        CleanComponentsState {
            coverage: false,
            profile: false,
            trace: false,
            cache: true,
        } => {
            vec![]
        }
        state => {
            panic!("Invalid state: {state:?}");
        }
    };

    test_runner(temp_dir).args(&args).assert().success();

    assert_eq!(check_clean_components_state(temp_dir.path()), state);
}

fn check_clean_components_state(path: &Path) -> CleanComponentsState {
    let scarb_metadata = ScarbCommand::metadata()
        .inherit_stderr()
        .current_dir(path)
        .no_deps()
        .run()
        .unwrap();

    let workspace_root = scarb_metadata.workspace.root;

    let packages_root: Vec<_> = scarb_metadata
        .packages
        .into_iter()
        .map(|package_metadata| package_metadata.root)
        .collect();

    CleanComponentsState {
        coverage: dirs_exist(&packages_root, COVERAGE_DIR),
        profile: dirs_exist(&packages_root, PROFILE_DIR),
        cache: dir_exists(&workspace_root, CACHE_DIR),
        trace: dir_exists(&workspace_root, TRACE_DIR),
    }
}

fn dirs_exist(root_dirs: &[Utf8PathBuf], dir_name: &str) -> bool {
    root_dirs
        .iter()
        .all(|root_dir| dir_exists(root_dir, dir_name))
}
fn dir_exists(dir: &Utf8PathBuf, dir_name: &str) -> bool {
    dir.join(dir_name).exists()
}
