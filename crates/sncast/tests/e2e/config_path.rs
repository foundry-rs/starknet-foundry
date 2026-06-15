use crate::helpers::runner::runner;
use configuration::test_utils::copy_config_to_tempdir;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use tempfile::tempdir;

#[test]
fn test_config_path_with_local() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec!["config-path"];

    let output = runner(&args).current_dir(tempdir.path()).assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
            Local Config:  [..]snfoundry.toml
            Global Config: [..]snfoundry.toml
        "},
    );
}

#[test]
fn test_config_path_with_missing_local() {
    let tempdir = tempdir().expect("Failed to create a temporary directory");
    let args = vec!["config-path"];

    let output = runner(&args).current_dir(tempdir.path()).assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
            Local Config:  missing
            Global Config: [..]snfoundry.toml
        "},
    );
}

#[test]
fn test_config_path_with_malformed_local() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_malformed.toml", None);
    let args = vec!["config-path"];

    let output = runner(&args).current_dir(tempdir.path()).assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
            Local Config:  [..]snfoundry.toml
            Global Config: [..]snfoundry.toml
        "},
    );
}

#[test]
fn test_config_path_json() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec!["--json", "config-path"];

    let output = runner(&args).current_dir(tempdir.path()).assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
            {"command":"config-path","global_config":"[..]snfoundry.toml","local_config":"[..]snfoundry.toml","type":"response"}
        "#},
    );
}

#[test]
fn test_config_path_json_with_missing_local() {
    let tempdir = tempdir().expect("Failed to create a temporary directory");
    let args = vec!["--json", "config-path"];

    let output = runner(&args).current_dir(tempdir.path()).assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
            {"command":"config-path","global_config":"[..]snfoundry.toml","local_config":null,"type":"response"}
        "#},
    );
}
