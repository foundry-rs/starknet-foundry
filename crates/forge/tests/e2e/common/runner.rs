use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use indoc::formatdoc;
use snapbox::cmd::{cargo_bin, Command as SnapboxCommand};
use std::env;
use std::process::Command;
use std::str::FromStr;

pub(crate) fn runner() -> SnapboxCommand {
    let snapbox = SnapboxCommand::new(cargo_bin!("snforge"));
    snapbox
}

pub(crate) fn setup_package(package_name: &str) -> TempDir {
    let temp = TempDir::new().unwrap();
    temp.copy_from(
        format!("tests/data/{package_name}"),
        &["**/*.cairo", "**/*.toml", "**/*.txt", "**/*.json"],
    )
    .unwrap();

    let snforge_std_path = Utf8PathBuf::from_str("../../snforge_std")
        .unwrap()
        .canonicalize_utf8()
        .unwrap()
        .to_string()
        .replace('\\', "/");

    let manifest_path = temp.child("Scarb.toml");
    manifest_path
        .write_str(&formatdoc!(
            r#"
                [package]
                name = "{}"
                version = "0.1.0"

                [[target.starknet-contract]]
                sierra = true
                casm = true

                [dependencies]
                starknet = "2.2.0"
                snforge_std = {{ path = "{}" }}
                "#,
            package_name,
            snforge_std_path
        ))
        .unwrap();

    temp
}

/// In context of GITHUB actions, get the repository name that triggered the workflow run.
/// Locally returns current branch.
///
/// `REPO_NAME` environment variable is expected to be in format `<repo_owner>/<repo_name>.git`.
pub(crate) fn get_remote_url() -> String {
    let name: &str = "REPO_NAME";
    if let Ok(v) = env::var(name) {
        v
    } else {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .output()
            .unwrap();

        String::from_utf8(output.stdout)
            .unwrap()
            .trim()
            .strip_prefix("git@github.com:")
            .unwrap()
            .to_string()
    }
}

/// In the context of GITHUB actions, get the source branch that triggered the workflow run.
/// Locally returns current branch.
pub(crate) fn get_current_branch() -> String {
    let name: &str = "BRANCH_NAME";
    match env::var(name) {
        Ok(v) => v,
        Err(_e) => {
            let output = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .output()
                .unwrap();

            String::from_utf8(output.stdout).unwrap()
        }
    }
}
