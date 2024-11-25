use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use indoc::formatdoc;
use shared::command::CommandExt;
use shared::test_utils::node_url::node_rpc_url;
use snapbox::cmd::{cargo_bin, Command as SnapboxCommand};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::{env, fs};
use test_utils::{get_assert_macros_version, tempdir_with_tool_versions};
use toml_edit::{value, DocumentMut};
use walkdir::WalkDir;

pub(crate) fn runner(temp_dir: &TempDir) -> SnapboxCommand {
    SnapboxCommand::new(snforge_test_bin_path()).current_dir(temp_dir)
}

// If ran on CI, we want to get the nextest's built binary
pub fn snforge_test_bin_path() -> PathBuf {
    if env::var("NEXTEST").unwrap_or("0".to_string()) == "1" {
        let snforge_nextest_env =
            env::var("NEXTEST_BIN_EXE_snforge").expect("No snforge binary for nextest found");
        return PathBuf::from(snforge_nextest_env);
    }
    cargo_bin!("snforge").to_path_buf()
}

pub(crate) fn test_runner(temp_dir: &TempDir) -> SnapboxCommand {
    runner(temp_dir).arg("test")
}

pub(crate) static BASE_FILE_PATTERNS: &[&str] = &["**/*.cairo", "**/*.toml"];

pub(crate) fn setup_package_with_file_patterns(
    package_name: &str,
    file_patterns: &[&str],
) -> TempDir {
    let temp = tempdir_with_tool_versions().unwrap();

    let is_from_docs_listings = fs::read_dir("../../docs/listings")
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .any(|entry| entry == package_name);

    let package_path = if is_from_docs_listings {
        format!("../../docs/listings/{package_name}",)
    } else {
        format!("tests/data/{package_name}",)
    };

    let package_path = Utf8PathBuf::from_str(&package_path)
        .unwrap()
        .canonicalize_utf8()
        .unwrap()
        .to_string()
        .replace('\\', "/");

    temp.copy_from(package_path, file_patterns).unwrap();

    let snforge_std_path = Utf8PathBuf::from_str("../../snforge_std")
        .unwrap()
        .canonicalize_utf8()
        .unwrap()
        .to_string()
        .replace('\\', "/");

    let manifest_path = temp.child("Scarb.toml");

    let mut scarb_toml = fs::read_to_string(&manifest_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();

    let is_workspace = scarb_toml.get("workspace").is_some();

    if is_workspace {
        scarb_toml["workspace"]["dependencies"]["snforge_std"]["path"] = value(snforge_std_path);
    } else {
        scarb_toml["dev-dependencies"]["snforge_std"]["path"] = value(snforge_std_path);
    }

    scarb_toml["dependencies"]["starknet"] = value("2.4.0");
    scarb_toml["dependencies"]["assert_macros"] =
        value(get_assert_macros_version().unwrap().to_string());
    scarb_toml["target.starknet-contract"]["sierra"] = value(true);

    if is_from_docs_listings {
        scarb_toml["dev-dependencies"]["snforge_std"]
            .as_table_mut()
            .and_then(|snforge_std| snforge_std.remove("workspace"));
    }

    manifest_path.write_str(&scarb_toml.to_string()).unwrap();

    // TODO (#2074): do that on .cairo.template files only
    replace_node_rpc_url_placeholders(temp.path());

    temp
}

pub(crate) fn setup_package(package_name: &str) -> TempDir {
    setup_package_with_file_patterns(package_name, BASE_FILE_PATTERNS)
}

fn replace_node_rpc_url_placeholders(dir_path: &Path) {
    let url = node_rpc_url();
    let temp_dir_files = WalkDir::new(dir_path);
    for entry in temp_dir_files {
        let entry = entry.unwrap();

        let path = entry.path();

        if path.is_file() {
            let content = fs::read_to_string(path).unwrap();

            let modified_content = content.replace("{{ NODE_RPC_URL }}", url.as_str());

            fs::write(path, modified_content).unwrap();
        }
    }
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

                [dev-dependencies]
                snforge_std.workspace = true
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
            .output_checked()
            .unwrap();

        String::from_utf8(output.stdout).unwrap().trim().to_string()
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
            .output_checked()
            .unwrap();

        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }
}
