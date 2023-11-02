use crate::forge_bench::collect_tests;
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use forge::{CrateLocation, TestCompilationTarget};
use std::path::PathBuf;
use std::str::FromStr;
use test_collector::LinkedLibrary;
use test_utils::corelib::corelib_path;

pub fn setup() -> (
    TestCompilationTarget,
    Vec<LinkedLibrary>,
    Utf8PathBuf,
    TempDir,
) {
    let package = collect_tests::setup();
    let path = Utf8PathBuf::from_path_buf(package.to_path_buf())
        .unwrap()
        .join("src");

    let snforge_std_path = PathBuf::from_str("../../snforge_std")
        .unwrap()
        .canonicalize()
        .unwrap();
    let linked_libraries = vec![
        LinkedLibrary {
            name: "simple_package".to_string(),
            path: PathBuf::from(path.clone()),
        },
        LinkedLibrary {
            name: "snforge_std".to_string(),
            path: snforge_std_path.join("src"),
        },
    ];

    let lib_content = std::fs::read_to_string(path.join("lib.cairo")).unwrap();
    let compilation_target = TestCompilationTarget {
        crate_root: path,
        crate_name: "simple_package".to_string(),
        crate_location: CrateLocation::Lib,
        lib_content,
    };

    (
        compilation_target,
        linked_libraries,
        corelib_path(),
        package,
    )
}

pub fn compile_tests(
    compilation_target: &TestCompilationTarget,
    linked_libraries: &[LinkedLibrary],
    corelib_path: &Utf8PathBuf,
) {
    compilation_target
        .compile_tests(linked_libraries, corelib_path)
        .unwrap();
}
