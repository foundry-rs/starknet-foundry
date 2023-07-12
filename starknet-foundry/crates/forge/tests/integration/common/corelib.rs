use assert_fs::fixture::PathCopy;
use assert_fs::TempDir;

pub fn corelib() -> TempDir {
    let corelib = TempDir::new().unwrap();
    corelib
        .copy_from("../../../cairo/corelib/src", &["**/*"])
        .unwrap();
    corelib
}
