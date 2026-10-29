use crate::helpers::runner::{Cast, runner};
use configuration::test_utils::copy_config_to_tempdir;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use std::fs;
use tempfile::tempdir;
use test_case::test_case;

#[test_case(false; "local")]
#[test_case(true; "global")]
fn test_alias_add_happy_path(global: bool) {
    let config_dir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let current_dir = tempdir().unwrap();
    let mut args = vec!["alias", "add", "foo", "0x43"];

    if global {
        args.push("--global");
        Cast::new()
            .config_dir(config_dir.path())
            .command()
            .args(&args)
            .current_dir(current_dir.path())
            .assert()
            .success();
    } else {
        runner(&args)
            .current_dir(config_dir.path())
            .assert()
            .success();
    }

    let contents = fs::read_to_string(config_dir.path().join("snfoundry.toml")).unwrap();
    assert!(contents.contains(indoc! {r#"
        [sncast.default.aliases]
        foo = "0x43"
    "#}));
}

#[test_case(false; "local")]
#[test_case(true; "global")]
fn test_alias_add_with_missing_config(global: bool) {
    let current_dir = tempdir().unwrap();
    let global_dir = tempdir().unwrap();
    let mut args = vec!["alias", "add", "foo", "0x43"];

    if global {
        args.push("--global");
        Cast::new()
            .config_dir(global_dir.path())
            .command()
            .args(&args)
            .current_dir(current_dir.path())
            .assert()
            .success();
    } else {
        runner(&args)
            .current_dir(current_dir.path())
            .assert()
            .success();
    }

    let config_dir = if global { &global_dir } else { &current_dir };
    let contents = fs::read_to_string(config_dir.path().join("snfoundry.toml")).unwrap();
    assert!(contents.contains(indoc! {r#"
        [sncast.default.aliases]
        foo = "0x43"
    "#}));
}

#[test_case(false; "without_force")]
#[test_case(true; "with_force")]
fn test_alias_add_overwrite_behavior(force: bool) {
    let current_dir = tempdir().unwrap();
    fs::write(
        current_dir.path().join("snfoundry.toml"),
        indoc! {r#"
            [sncast.default.aliases]
            foo = "0x1"
        "#},
    )
    .unwrap();

    let mut args = vec!["alias", "add", "foo", "0x2"];
    if force {
        args.push("--force");
    }

    if force {
        runner(&args)
            .current_dir(current_dir.path())
            .assert()
            .success();
    } else {
        let output = runner(&args)
            .current_dir(current_dir.path())
            .assert()
            .failure();
        assert_stderr_contains(
            output,
            "Error: Alias `foo` already exists. Use --force to overwrite it.",
        );
    }

    let expected_value = if force { "0x2" } else { "0x1" };
    assert_eq!(
        fs::read_to_string(current_dir.path().join("snfoundry.toml")).unwrap(),
        format!("[sncast.default.aliases]\nfoo = \"{expected_value}\"\n")
    );
}

#[test]
fn test_added_alias_is_listed() {
    let current_dir = tempdir().unwrap();

    runner(&["alias", "add", "foo", "0x43"])
        .current_dir(current_dir.path())
        .assert()
        .success();

    let output = runner(&["alias", "list"])
        .current_dir(current_dir.path())
        .assert()
        .success();

    assert_stdout_contains(output, "foo: 0x43");
}
