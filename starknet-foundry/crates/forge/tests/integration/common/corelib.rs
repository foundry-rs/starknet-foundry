use assert_fs::fixture::PathCopy;
use assert_fs::TempDir;

pub fn corelib() -> TempDir {
    let corelib = TempDir::new().unwrap();
    corelib
        .copy_from("../../../corelib/src", &["**/*"])
        .unwrap();
    corelib
}

pub fn predeployed_contracts() -> TempDir {
    let predeployed_contracts = TempDir::new().unwrap();
    predeployed_contracts
        .copy_from(
            "../../crates/cheatable-starknet/predeployed-contracts",
            &["**/*"],
        )
        .unwrap();
    predeployed_contracts
}
