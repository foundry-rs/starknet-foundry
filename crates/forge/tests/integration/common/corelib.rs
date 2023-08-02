use assert_fs::fixture::{FileTouch, FileWriteStr, PathChild, PathCopy};
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use forge::scarb::extract_metadata_from_package;
use indoc::indoc;
use scarb_metadata::MetadataCommand;
use test_collector::LinkedLibrary;

#[allow(clippy::module_name_repetitions)]
pub fn corelib_path() -> Utf8PathBuf {
    extract_corelib_and_cheatcodes().0
}

pub fn cheatcodes() -> LinkedLibrary {
    extract_corelib_and_cheatcodes().1
}

fn extract_corelib_and_cheatcodes() -> (Utf8PathBuf, LinkedLibrary) {
    // create an empty scarb project with cheatcodes dependency
    // to extract cheatcodes location from metadata
    let dir = TempDir::new().unwrap();

    let lib_file = dir.child("src/lib.cairo");
    lib_file.touch().unwrap();

    let manifest_file = dir.child("Scarb.toml");
    manifest_file.touch().unwrap();
    manifest_file
        // TODO XD
        .write_str(indoc!(
            r#"
            [package]
            name = "pkg"
            version = "0.1.0"
            [dependencies]
            "#
        ))
        .unwrap();

    let scarb_metadata = MetadataCommand::new()
        .current_dir(dir.to_path_buf())
        .inherit_stderr()
        .exec()
        .unwrap();
    let package = &scarb_metadata.workspace.members[0];
    let (_, _, corelib_path, dependencies, _) =
        extract_metadata_from_package(&scarb_metadata, package).unwrap();

    let cheatcodes = dependencies
        .iter()
        .find(|dep| dep.name == "cheatcodes")
        .unwrap()
        .clone();

    (corelib_path, cheatcodes)
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
