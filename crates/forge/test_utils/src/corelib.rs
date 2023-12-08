use assert_fs::fixture::{PathCopy};
use assert_fs::TempDir;




#[must_use]
pub fn predeployed_contracts() -> TempDir {
    let predeployed_contracts = TempDir::new().unwrap();
    predeployed_contracts
        .copy_from("../../crates/cheatnet/predeployed-contracts", &["**/*"])
        .unwrap();
    predeployed_contracts
}
