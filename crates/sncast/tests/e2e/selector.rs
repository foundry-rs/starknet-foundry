use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};

#[test]
fn test_selector_happy_case() {
    let args = vec!["utils", "selector", "transfer"];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
            Selector: 0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e
        "},
    );
}

#[test]
fn test_selector_json_output() {
    let args = vec!["--json", "utils", "selector", "transfer"];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();
    let stdout = output.get_output().stdout.clone();

    let json: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(json["command"], "selector");
    assert_eq!(json["type"], "response");
    assert_eq!(
        json["selector"],
        "0x0083afd3f4caedc6eebf44246fe54e38c95e3179a5ec9ea81740eca5b482d12e"
    );
}

#[test]
fn test_selector_with_parentheses() {
    let args = vec!["utils", "selector", "transfer(u256)"];
    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
            Error: Parentheses and the content within should not be supplied
        "},
    );
}
