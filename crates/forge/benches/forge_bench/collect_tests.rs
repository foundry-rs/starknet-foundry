use assert_fs::fixture::PathCopy;
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use forge::collect_test_compilation_targets;

pub fn setup() -> TempDir {
    let temp = TempDir::new().unwrap();
    temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    temp
}

pub fn collect_tests(package: &TempDir) {
    let path = Utf8PathBuf::from_path_buf(package.to_path_buf()).unwrap();

    let _ = collect_test_compilation_targets(&path, "simple_package", &path);
}
