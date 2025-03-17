#[cfg(feature = "smoke")]
use super::common::runner::{get_current_branch, get_remote_url};

use super::common::runner::{runner, snforge_test_bin_path, test_runner};
use assert_fs::TempDir;
use assert_fs::fixture::{FileTouch, PathChild};
use forge::CAIRO_EDITION;
use forge::scarb::config::SCARB_MANIFEST_TEMPLATE_CONTENT;
use indoc::{formatdoc, indoc};
use shared::test_utils::output_assert::assert_stdout_contains;
use snapbox::assert_matches;
use snapbox::cmd::Command as SnapboxCommand;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs, iter};
use test_utils::{get_local_snforge_std_absolute_path, tempdir_with_tool_versions};
use toml_edit::{DocumentMut, Formatted, InlineTable, Item, Value};

#[test]
fn init_new_project() {
    let temp = tempdir_with_tool_versions().unwrap();

    let output = runner(&temp)
        .args(["init", "test_name"])
        .env("DEV_DISABLE_SNFORGE_STD_DEPENDENCY", "true")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc!(
            r"
                [WARNING] Command `snforge init` is deprecated and will be removed in the future. Please use `snforge new` instead.
            "
        ),
    );

    validate_init(&temp.join("test_name"), false);
}

#[test]
fn create_new_project_dir_not_exist() {
    let temp = tempdir_with_tool_versions().unwrap();
    let project_path = temp.join("new").join("project");

    runner(&temp)
        .args(["new", "--name", "test_name"])
        .arg(&project_path)
        .env("DEV_DISABLE_SNFORGE_STD_DEPENDENCY", "true")
        .assert()
        .success();

    validate_init(&project_path, false);
}

#[test]
fn create_new_project_dir_not_empty() {
    let temp = tempdir_with_tool_versions().unwrap();
    temp.child("empty.txt").touch().unwrap();

    let output = runner(&temp)
        .args(["new", "--name", "test_name"])
        .arg(temp.path())
        .assert()
        .code(2);

    assert_stdout_contains(
        output,
        indoc!(
            r"
                [ERROR] The provided path [..] points to a non-empty directory. If you wish to create a project in this directory, use the `--overwrite` flag
            "
        ),
    );
}

#[test]
fn create_new_project_dir_exists_and_empty() {
    let temp = tempdir_with_tool_versions().unwrap();
    let project_path = temp.join("new").join("project");

    fs::create_dir_all(&project_path).unwrap();
    assert!(project_path.exists());

    runner(&temp)
        .args(["new", "--name", "test_name"])
        .arg(&project_path)
        .env("DEV_DISABLE_SNFORGE_STD_DEPENDENCY", "true")
        .assert()
        .success();

    validate_init(&project_path, false);
}

#[test]
fn init_new_project_from_scarb() {
    let temp = tempdir_with_tool_versions().unwrap();

    SnapboxCommand::new("scarb")
        .current_dir(&temp)
        .args(["new", "test_name"])
        .env("SCARB_INIT_TEST_RUNNER", "starknet-foundry")
        .env(
            "PATH",
            append_to_path_var(snforge_test_bin_path().parent().unwrap()),
        )
        .assert()
        .success();

    validate_init(&temp.join("test_name"), true);
}

pub fn append_to_path_var(path: &Path) -> OsString {
    let script_path = iter::once(path.to_path_buf());
    let os_path = env::var_os("PATH").unwrap();
    let other_paths = env::split_paths(&os_path);
    env::join_paths(script_path.chain(other_paths)).unwrap()
}

fn validate_init(project_path: &PathBuf, validate_snforge_std: bool) {
    let manifest_path = project_path.join("Scarb.toml");
    let scarb_toml = fs::read_to_string(manifest_path.clone()).unwrap();

    let snforge_std_assert = if validate_snforge_std {
        "\nsnforge_std = \"[..]\""
    } else {
        ""
    };

    let expected = formatdoc!(
        r#"
            [package]
            name = "test_name"
            version = "0.1.0"
            edition = "{CAIRO_EDITION}"

            # See more keys and their definitions at https://docs.swmansion.com/scarb/docs/reference/manifest.html

            [dependencies]
            starknet = "[..]"

            [dev-dependencies]{}
            assert_macros = "[..]"

            [[target.starknet-contract]]
            sierra = true

            [scripts]
            test = "snforge test"

            [tool.scarb]
            allow-prebuilt-plugins = ["snforge_std"]
            {SCARB_MANIFEST_TEMPLATE_CONTENT}
        "#,
        snforge_std_assert
    ).trim_end()
    .to_string()+ "\n";

    assert_matches(&expected, &scarb_toml);

    let mut scarb_toml = DocumentMut::from_str(&scarb_toml).unwrap();

    let dependencies = scarb_toml
        .get_mut("dev-dependencies")
        .unwrap()
        .as_table_mut()
        .unwrap();

    let local_snforge_std = get_local_snforge_std_absolute_path()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let mut snforge_std = InlineTable::new();
    snforge_std.insert("path", Value::String(Formatted::new(local_snforge_std)));

    dependencies.remove("snforge_std");
    dependencies.insert("snforge_std", Item::Value(Value::InlineTable(snforge_std)));

    std::fs::write(manifest_path, scarb_toml.to_string()).unwrap();

    let output = test_runner(&TempDir::new().unwrap())
        .current_dir(project_path)
        .assert()
        .success();

    let expected = indoc!(
        r"
        [..]Compiling[..]
        [..]Finished[..]

        Collected 2 test(s) from test_name package
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [PASS] test_name_integrationtest::test_contract::test_increase_balance [..]
        [PASS] test_name_integrationtest::test_contract::test_cannot_increase_balance_with_zero_value [..]
        Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
        "
    );

    assert_stdout_contains(output, expected);
}

#[test]
#[cfg(feature = "smoke")]
fn test_init_project_with_custom_snforge_dependency_git() {
    let temp = tempdir_with_tool_versions().unwrap();

    runner(&temp)
        .args(["new", "test_name"])
        .env("DEV_DISABLE_SNFORGE_STD_DEPENDENCY", "true")
        .assert()
        .success();

    let project_path = temp.join("test_name");
    let manifest_path = project_path.join("Scarb.toml");

    let scarb_toml = std::fs::read_to_string(&manifest_path).unwrap();
    let mut scarb_toml = DocumentMut::from_str(&scarb_toml).unwrap();

    let dependencies = scarb_toml
        .get_mut("dev-dependencies")
        .unwrap()
        .as_table_mut()
        .unwrap();

    let branch = get_current_branch();
    let remote_url = format!("https://github.com/{}", get_remote_url());

    let mut snforge_std = InlineTable::new();
    snforge_std.insert("git", Value::String(Formatted::new(remote_url.clone())));
    snforge_std.insert("branch", Value::String(Formatted::new(branch)));

    dependencies.remove("snforge_std");
    dependencies.insert("snforge_std", Item::Value(Value::InlineTable(snforge_std)));

    std::fs::write(&manifest_path, scarb_toml.to_string()).unwrap();

    let output = test_runner(&temp)
        .current_dir(&project_path)
        .assert()
        .success();

    let expected = formatdoc!(
        r"
        [..]Updating git repository {}
        [..]Compiling test_name v0.1.0[..]
        [..]Finished[..]

        Collected 2 test(s) from test_name package
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [PASS] test_name_integrationtest::test_contract::test_increase_balance [..]
        [PASS] test_name_integrationtest::test_contract::test_cannot_increase_balance_with_zero_value [..]
        Tests: 2 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
        ",
        remote_url.trim_end_matches(".git")
    );

    assert_stdout_contains(output, expected);
}

#[test]
fn create_new_project_and_check_gitignore() {
    let temp = tempdir_with_tool_versions().unwrap();
    let project_path = temp.join("project");

    runner(&temp)
        .env("DEV_DISABLE_SNFORGE_STD_DEPENDENCY", "true")
        .args(["new", "--name", "test_name"])
        .arg(&project_path)
        .assert()
        .success();

    let gitignore_path = project_path.join(".gitignore");
    assert!(gitignore_path.exists(), ".gitignore file should exist");

    let gitignore_content = fs::read_to_string(gitignore_path).unwrap();

    let expected_gitignore_content = indoc! {
        r"
        target
        .snfoundry_cache/
        snfoundry_trace/
        coverage/
        profile/
        "
    };

    assert_eq!(gitignore_content, expected_gitignore_content);
}
