#[cfg(feature = "smoke")]
use super::common::runner::{get_current_branch, get_remote_url};

use super::common::runner::{runner, snforge_test_bin_path, test_runner};
use assert_fs::TempDir;
use assert_fs::fixture::{FileTouch, PathChild};
use forge::CAIRO_EDITION;
use forge::Template;
use forge::scarb::config::SCARB_MANIFEST_TEMPLATE_CONTENT;
use indoc::{formatdoc, indoc};
use regex::Regex;
use shared::consts::FREE_RPC_PROVIDER_URL;
use shared::test_utils::output_assert::assert_stdout_contains;
use snapbox::assert_matches;
use snapbox::cmd::Command as SnapboxCommand;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::LazyLock;
use std::{env, fs, iter};
use test_case::test_case;
use test_utils::{get_local_snforge_std_absolute_path, tempdir_with_tool_versions};
use toml_edit::{DocumentMut, Formatted, InlineTable, Item, Value};

static RE_NEWLINES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());

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

    validate_init(&temp.join("test_name"), None, &Template::BalanceContract);
}

#[test_case(&Template::CairoProgram; "cairo-program")]
#[test_case(&Template::BalanceContract; "balance-contract")]
#[test_case(&Template::Erc20Contract; "erc20-contract")]
fn create_new_project_dir_not_exist(template: &Template) {
    let temp = tempdir_with_tool_versions().unwrap();
    let project_path = temp.join("new").join("project");

    runner(&temp)
        .args([
            "new",
            "--name",
            "test_name",
            "--template",
            template.to_string().as_str(),
        ])
        .arg(&project_path)
        .env("DEV_DISABLE_SNFORGE_STD_DEPENDENCY", "true")
        .assert()
        .success();

    validate_init(&project_path, None, template);
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

    validate_init(&project_path, None, &Template::BalanceContract);
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

    validate_init(
        &temp.join("test_name"),
        Some(SnforgeStd::Normal),
        &Template::BalanceContract,
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_2_9_1"), ignore)]
fn init_new_project_from_scarb_snforge_std_compatibility() {
    let temp = tempdir_with_tool_versions().unwrap();
    let tool_version_path = temp.join(".tool-versions");
    fs::write(tool_version_path, "scarb 2.11.4").unwrap();

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

    validate_init(
        &temp.join("test_name"),
        Some(SnforgeStd::Compatibility),
        &Template::BalanceContract,
    );
}
pub fn append_to_path_var(path: &Path) -> OsString {
    let script_path = iter::once(path.to_path_buf());
    let os_path = env::var_os("PATH").unwrap();
    let other_paths = env::split_paths(&os_path);
    env::join_paths(script_path.chain(other_paths)).unwrap()
}

fn validate_init(
    project_path: &PathBuf,
    validate_snforge_std: Option<SnforgeStd>,
    template: &Template,
) {
    let manifest_path = project_path.join("Scarb.toml");
    let scarb_toml = fs::read_to_string(manifest_path.clone()).unwrap();

    let expected = get_expected_manifest_content(template, validate_snforge_std);
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

    fs::write(manifest_path, scarb_toml.to_string()).unwrap();

    let output = test_runner(&TempDir::new().unwrap())
        .current_dir(project_path)
        .assert()
        .success();

    let expected = get_expected_output(template);
    assert_stdout_contains(output, expected);
}

enum SnforgeStd {
    Normal,
    Compatibility,
}

fn get_expected_manifest_content(
    template: &Template,
    validate_snforge_std: Option<SnforgeStd>,
) -> String {
    let snforge_std_assert = match validate_snforge_std {
        None => "",
        Some(snforge_std) => match snforge_std {
            SnforgeStd::Normal => "\nsnforge_std = \"[..]\"",
            SnforgeStd::Compatibility => "\nsnforge_std_compatibility = \"[..]\"",
        },
    };

    let target_contract_entry = "[[target.starknet-contract]]\nsierra = true";

    let fork_config = if let Template::Erc20Contract = template {
        &formatdoc!(
            r#"
            [[tool.snforge.fork]]
            name = "SEPOLIA_LATEST"
            url = "{FREE_RPC_PROVIDER_URL}"
            block_id = {{ tag = "latest" }}
        "#
        )
    } else {
        ""
    };

    let (dependencies, target_contract_entry) = match template {
        Template::BalanceContract => ("starknet = \"[..]\"", target_contract_entry),
        Template::Erc20Contract => (
            "openzeppelin_token = \"[..]\"\nstarknet = \"[..]\"",
            target_contract_entry,
        ),
        Template::CairoProgram => ("", ""),
    };

    let expected_manifest = formatdoc!(
        r#"
            [package]
            name = "test_name"
            version = "0.1.0"
            edition = "{CAIRO_EDITION}"

            # See more keys and their definitions at https://docs.swmansion.com/scarb/docs/reference/manifest.html

            [dependencies]
            {dependencies}

            [dev-dependencies]{}
            assert_macros = "[..]"

            {target_contract_entry}

            [scripts]
            test = "snforge test"

            [tool.scarb]
            allow-prebuilt-plugins = ["snforge_std"]

            {fork_config}

            {}
        "#,
        snforge_std_assert,
        SCARB_MANIFEST_TEMPLATE_CONTENT.trim_end()
    );

    // Replace 3 or more consecutive newlines with exactly 2 newlines
    RE_NEWLINES
        .replace_all(&expected_manifest, "\n\n")
        .to_string()
}

fn get_expected_output(template: &Template) -> &str {
    match template {
        Template::CairoProgram => {
            indoc!(
                r"
                [..]Compiling[..]
                [..]Finished[..]

                Collected 1 test(s) from test_name package
                Running 1 test(s) from src/
                [PASS] test_name::tests::it_works [..]
                Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
                "
            )
        }
        Template::BalanceContract => {
            indoc!(
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
            )
        }
        Template::Erc20Contract => {
            indoc!(
                r"
                [..]Compiling[..]
                [..]Finished[..]

                Collected 8 test(s) from test_name package
                Running 8 test(s) from tests/
                [PASS] test_name_integrationtest::test_erc20::should_panic_transfer [..]
                [PASS] test_name_integrationtest::test_erc20::test_get_balance [..]
                [PASS] test_name_integrationtest::test_erc20::test_transfer [..]
                [PASS] test_name_integrationtest::test_erc20::test_transfer_event [..]
                [PASS] test_name_integrationtest::test_token_sender::test_multisend [..]
                [PASS] test_name_integrationtest::test_token_sender::test_single_send_fuzz [..]
                [PASS] test_name_integrationtest::test_token_sender::test_single_send [..]
                [PASS] test_name_integrationtest::test_erc20::test_fork_transfer [..]
                Running 0 test(s) from src/
                Tests: 8 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
                "
            )
        }
    }
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
