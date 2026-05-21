use crate::helpers::constants::URL;
use crate::helpers::runner::runner;
use indoc::indoc;

#[tokio::test]
async fn test_happy_case() {
    let args = vec!["get", "spec-version", "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Spec version retrieved

        Spec Version: [..]
    "});
}

#[tokio::test]
async fn test_happy_case_json() {
    let args = vec!["--json", "get", "spec-version", "--url", URL];
    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
        {"command":"get spec-version","spec_version":"[..]","type":"response"}
    "#});
}
