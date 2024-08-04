use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use fs_extra::dir::{copy, CopyOptions};
use indoc::formatdoc;
use shared::command::CommandExt;
use shared::test_utils::node_url::node_rpc_url;
use snapbox::cmd::{cargo_bin, Command as SnapboxCommand};
use std::fs::{create_dir_all, remove_dir_all};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::sync::LazyLock;
use std::{env, fs};
use test_utils::tempdir_with_tool_versions;
use toml_edit::{value, DocumentMut};
use walkdir::WalkDir;

/// To avoid rebuilding `snforge_std` and associated plugin for each test, we cache it in a directory and copy it to the e2e test temp directory.
static BASE_CACHE_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| init_base_cache_dir().expect("Failed to initialize base cache directory"));

fn init_base_cache_dir() -> anyhow::Result<PathBuf> {
    let cache_dir_path = env::current_dir()?.join("forge_e2e_cache");
    if cache_dir_path.exists() {
        remove_dir_all(&cache_dir_path)?;
    }
    create_dir_all(&cache_dir_path)?;
    let cache_dir_path = cache_dir_path.canonicalize()?;

    let snforge_std = PathBuf::from("../../snforge_std").canonicalize()?;

    SnapboxCommand::new("scarb")
        .arg("build")
        .current_dir(snforge_std.as_path())
        .env("SCARB_CACHE", &cache_dir_path)
        .assert()
        .success();

    Ok(cache_dir_path)
}

pub(crate) fn runner(temp_dir: &TempDir) -> SnapboxCommand {
    copy(
        BASE_CACHE_DIR.as_path(),
        temp_dir.path(),
        &CopyOptions::new().overwrite(true).content_only(true),
    )
    .unwrap();

    SnapboxCommand::new(cargo_bin!("snforge"))
        .env("SCARB_CACHE", temp_dir.path())
        .current_dir(temp_dir)
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
        .parse::<DocumentMut>()
        .unwrap();
    scarb_toml["dev-dependencies"]["snforge_std"]["path"] = value(snforge_std_path);
    scarb_toml["dependencies"]["starknet"] = value("2.4.0");
    scarb_toml["target.starknet-contract"]["sierra"] = value(true);

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
            .output_checked()
            .unwrap();

        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }
}
