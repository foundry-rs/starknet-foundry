use configuration::test_utils::copy_config_to_tempdir;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use std::fs;
use tempfile::tempdir;

use crate::helpers::runner::runner;

#[test]
fn test_alias_list_happy_case() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    let args = vec!["alias", "list"];

    let output = runner(&args).current_dir(tempdir.path()).assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
            map:       0xcd8f9ab31324bb93251837e4efb4223ee195454f6304fcfcb277e277653008
            map-class: 0x2a09379665a749e609b4a8459c86fe954566a6beeaddd0950e43f6c700ed321
        "},
    );
}

#[test]
fn test_alias_list_empty() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec!["alias", "list"];

    let output = runner(&args).current_dir(tempdir.path()).assert().success();

    assert_stdout_contains(
        output,
        "No aliases configured. See aliases guide: https://foundry-rs.github.io/starknet-foundry/starknet/aliases.html",
    );
}

#[test]
fn test_alias_list_json() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    let args = vec!["--json", "alias", "list"];

    let output = runner(&args).current_dir(tempdir.path()).assert().success();

    assert_stdout_contains(
        output,
        r#"{"aliases":[{"name":"map","value":"0xcd8f9ab31324bb93251837e4efb4223ee195454f6304fcfcb277e277653008"},{"name":"map-class","value":"0x2a09379665a749e609b4a8459c86fe954566a6beeaddd0950e43f6c700ed321"}],"command":"alias list","type":"response"}"#,
    );
}

#[test]
fn test_alias_list_invalid_alias_value() {
    let tempdir = tempdir().unwrap();
    fs::write(
        tempdir.path().join("snfoundry.toml"),
        indoc! {r#"
            [sncast.default]
            url = "http://127.0.0.1:5055/rpc"

            [sncast.default.aliases]
            map = "not-a-felt"
        "#},
    )
    .unwrap();

    let output = runner(&["alias", "list"])
        .current_dir(tempdir.path())
        .assert()
        .failure();

    assert_stderr_contains(
        output,
        indoc! {r"
            Command: alias list
            Error: Failed to load local config at [..]snfoundry.toml

            Caused by:
                Failed to parse field `aliases`: Failed to parse `not-a-felt` to felt
        "},
    );
}
