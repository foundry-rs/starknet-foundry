use assert_fs::fixture::{FileTouch, FileWriteStr, PathChild, PathCopy};
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use indoc::indoc;
use scarb_metadata::MetadataCommand;

#[must_use]
pub fn predeployed_contracts() -> TempDir {
    let predeployed_contracts = TempDir::new().unwrap();
    predeployed_contracts
        .copy_from("../../crates/cheatnet/predeployed-contracts", &["**/*"])
        .unwrap();
    predeployed_contracts
}
