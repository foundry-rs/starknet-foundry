use ark_std::iterable::Iterable;
use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use indoc::formatdoc;
use regex::Regex;
use snapbox::cmd::{cargo_bin, Command as SnapboxCommand};
use std::env;
use std::process::Command;
use std::str::FromStr;

pub(crate) fn runner() -> SnapboxCommand {
    let snapbox = SnapboxCommand::new(cargo_bin!("snforge"));
    snapbox
}

pub(crate) static BASE_FILE_PATTERNS: &[&str] = &["**/*.cairo", "**/*.toml"];

pub(crate) fn setup_package_with_file_patterns(
    package_name: &str,
    file_patterns: &[&str],
) -> TempDir {
    let temp = TempDir::new().unwrap();
    temp.copy_from(format!("tests/data/{package_name}"), file_patterns)
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

pub(crate) fn setup_package(package_name: &str) -> TempDir {
    setup_package_with_file_patterns(package_name, BASE_FILE_PATTERNS)
}

pub(crate) fn setup_hello_workspace() -> TempDir {
    let temp = TempDir::new().unwrap();
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
                starknet = "2.2.0"
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
    let temp = TempDir::new().unwrap();
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
                starknet = "2.2.0"
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

pub(crate) fn find_with_wildcard(line: &str, actual: &Vec<String>) -> Option<usize> {
    let escaped = regex::escape(line);
    let replaced = escaped.replace("\\[\\.\\.\\]", ".*");
    let wrapped = format!("^{replaced}$");
    let re = Regex::new(wrapped.as_str()).unwrap();

    actual.iter().position(|other| re.is_match(other))
}

pub(crate) fn is_present(line: &str, actual: &mut Vec<String>) -> bool {
    let position = find_with_wildcard(line, actual);
    if let Some(position) = position {
        actual.remove(position);
        return true;
    }
    false
}

pub(crate) fn assert_output_contains(output: &str, lines: &str) {
    let asserted_lines: Vec<String> = lines.lines().map(std::convert::Into::into).collect();
    let mut actual_lines: Vec<String> = output.lines().map(std::convert::Into::into).collect();

    let mut matches = true;
    let mut out = String::new();

    for line in &asserted_lines {
        if is_present(line, &mut actual_lines) {
            out.push_str("| ");
        } else {
            matches = false;
            out.push_str("- ");
        }
        out.push_str(line);
        out.push('\n');
    }
    for remaining_line in actual_lines {
        matches = false;
        out.push_str("+ ");
        out.push_str(&remaining_line);
        out.push('\n');
    }

    assert!(matches, "Stdout does not match:\n\n{out}");
}

#[macro_export]
macro_rules! assert_stdout_contains {
    ( $output:expr, $lines:expr ) => {{
        use $crate::e2e::common::runner::assert_output_contains;

        let output = $output.get_output();
        let stdout = String::from_utf8(output.stdout.clone()).unwrap();

        assert_output_contains(&stdout, $lines);
    }};
}

#[macro_export]
macro_rules! assert_stderr_contains {
    ( $output:expr, $lines:expr ) => {{
        use $crate::e2e::common::runner::assert_output_contains;

        let output = $output.get_output();
        let stderr = String::from_utf8(output.stderr.clone()).unwrap();

        assert_output_contains(&stderr, $lines);
    }};
}
