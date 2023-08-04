use assert_fs::fixture::{FileTouch, FileWriteStr, PathChild, PathCopy};
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use forge::scarb::extract_metadata_from_package;
use indoc::indoc;
use scarb_metadata::MetadataCommand;

#[allow(clippy::module_name_repetitions)]
pub fn corelib_path() -> Utf8PathBuf {
    // create an empty scarb project to extract corelib location from metadata
    let dir = TempDir::new().unwrap();

    let lib_file = dir.child("src/lib.cairo");
    lib_file.touch().unwrap();

    let manifest_file = dir.child("Scarb.toml");
    manifest_file.touch().unwrap();
    manifest_file
        .write_str(indoc!(
            r#"
            [package]
            name = "pkg"
            version = "0.1.0"
            "#
        ))
        .unwrap();

    let scarb_metadata = MetadataCommand::new()
        .current_dir(dir.to_path_buf())
        .inherit_stderr()
        .exec()
        .unwrap();
    let package = &scarb_metadata.workspace.members[0];
    let (_, _, corelib_path, _, _) =
        extract_metadata_from_package(&scarb_metadata, package).unwrap();

    corelib_path
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
