use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use indoc::formatdoc;
use snapbox::cmd::{cargo_bin, Command as SnapboxCommand};
use std::process::Command;
use std::str::FromStr;
use std::{env, fs};
use test_utils::tempdir_with_tool_versions;
use toml_edit::{value, Document};

pub(crate) fn runner() -> SnapboxCommand {
    SnapboxCommand::new(cargo_bin!("snforge"))
}

pub(crate) fn test_runner() -> SnapboxCommand {
    runner().arg("test")
}

pub(crate) static BASE_FILE_PATTERNS: &[&str] = &["**/*.cairo", "**/*.toml"];

pub(crate) fn setup_package_with_file_patterns(
    package_name: &str,
    file_patterns: &[&str],
) -> TempDir {
    let temp = tempdir_with_tool_versions().unwrap();
    temp.copy_from(format!("tests/data/{package_name}"), file_patterns)
        .unwrap();

    let snforge_std_path = Utf8PathBuf::from_str("../../snforge_std")
        .unwrap()
        .canonicalize_utf8()
        .unwrap()
        .to_string()
        .replace('\\', "/");

    let manifest_path = temp.child("Scarb.toml");

    let mut scarb_toml = fs::read_to_string(&manifest_path)
        .unwrap()
        .parse::<Document>()
        .unwrap();
    scarb_toml["dependencies"]["snforge_std"]["path"] = value(snforge_std_path);
    scarb_toml["dependencies"]["starknet"] = value("2.4.0");
    scarb_toml["target.starknet-contract"]["sierra"] = value(true);

    manifest_path.write_str(&scarb_toml.to_string()).unwrap();

    temp
}

pub(crate) fn setup_package(package_name: &str) -> TempDir {
    setup_package_with_file_patterns(package_name, BASE_FILE_PATTERNS)
}

pub(crate) fn setup_hello_workspace() -> TempDir {
    let temp = tempdir_with_tool_versions().unwrap();
    temp.copy_from("tests/data/hello_workspaces", &["**/*.cairo", "**/*.toml"])
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
                [workspace]
                members = [
                    "crates/*",
                ]
                
                [workspace.scripts]
                test = "snforge"
                
                [workspace.tool.snforge]

                
                [workspace.dependencies]
                starknet = "2.4.0"
                snforge_std = {{ path = "{}" }}
                
                [workspace.package]
                version = "0.1.0"
                
                [package]
                name = "hello_workspaces"
                version.workspace = true
                
                [scripts]
                test.workspace = true
                
                [tool]
                snforge.workspace = true
                
                [dependencies]
                starknet.workspace = true
                fibonacci = {{ path = "crates/fibonacci" }}
                addition = {{ path = "crates/addition" }}
                
                [[target.starknet-contract]]
                sierra = true
                casm = true
                "#,
            snforge_std_path
        ))
        .unwrap();

    temp
}

pub(crate) fn setup_virtual_workspace() -> TempDir {
    let temp = tempdir_with_tool_versions().unwrap();
    temp.copy_from("tests/data/virtual_workspace", &["**/*.cairo", "**/*.toml"])
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
                [workspace]
                members = [
                    "dummy_name/*",
                ]
                
                [workspace.scripts]
                test = "snforge"
                
                [workspace.tool.snforge]
                
                [workspace.dependencies]
                starknet = "2.4.0"
                snforge_std = {{ path = "{}" }}
                
                [workspace.package]
                version = "0.1.0"
                
                [scripts]
                test.workspace = true
                
                [tool]
                snforge.workspace = true
                
                [[target.starknet-contract]]
                sierra = true
                casm = true
                "#,
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
    if let Ok(v) = env::var(name) {
        v
    } else {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .unwrap();

        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }
}
