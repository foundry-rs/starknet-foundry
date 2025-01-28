use super::common::runner::{runner, setup_package, test_runner};
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use scarb_api::metadata::MetadataCommandExt;
use scarb_api::ScarbCommand;
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

    let clean_components_state: CleanComponentsState = CleanComponentsState {
        coverage: false,
        profile: false,
        cache: true,
        trace: false,
    };

    assert_eq!(
        check_clean_components_state(temp_dir.path()),
        clean_components_state
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

    let clean_components_state: CleanComponentsState = CleanComponentsState {
        coverage: false,
        profile: false,
        cache: false,
        trace: false,
    };

    assert_eq!(
        check_clean_components_state(temp_dir.path()),
        clean_components_state
    );
}

#[test]
fn test_clean_all() {
    let temp_dir = setup_package("coverage_project");

    let clean_components_state = CleanComponentsState {
        coverage: true,
        profile: true,
        cache: true,
        trace: true,
    };

    generate_clean_components(clean_components_state, &temp_dir);

    runner(&temp_dir).arg("clean").arg("all").assert().success();

    let clean_components_state: CleanComponentsState = CleanComponentsState {
        coverage: false,
        profile: false,
        cache: false,
        trace: false,
    };

    assert_eq!(
        check_clean_components_state(temp_dir.path()),
        clean_components_state
    );
}

fn generate_clean_components(clean_components_state: CleanComponentsState, temp_dir: &TempDir) {
    if clean_components_state.coverage {
        assert!(clean_components_state.trace && clean_components_state.cache);
        test_runner(temp_dir).arg("--coverage").assert().success();
    } else if clean_components_state.profile {
        assert!(clean_components_state.trace && clean_components_state.cache);
        test_runner(temp_dir)
            .arg("--build-profile")
            .assert()
            .success();
    } else if clean_components_state.trace {
        assert!(clean_components_state.cache);
        test_runner(temp_dir)
            .arg("--save-trace-data")
            .assert()
            .success();
    } else if clean_components_state.cache {
        test_runner(temp_dir).assert().success();
    }

    assert_eq!(
        check_clean_components_state(temp_dir.path()),
        clean_components_state
    );
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
        .map(|package_metada| package_metada.root)
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
