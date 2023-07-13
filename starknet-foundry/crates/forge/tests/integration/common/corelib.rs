use assert_fs::fixture::PathCopy;
use assert_fs::TempDir;

pub fn corelib() -> TempDir {
    let corelib = TempDir::new().unwrap();
    corelib
        .copy_from("../../../cairo/corelib/src", &["**/*"])
        .unwrap();
    corelib
}

pub fn predeployed_contracts() -> TempDir {
    let corelib = TempDir::new().unwrap();
    corelib
        .copy_from(
            "../../crates/cheatable-starknet/predeployed-contracts",
            &["**/*"],
        )
        .unwrap();
    corelib
}
