use crate::helpers::constants::{
    DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA, MAP_CONTRACT_ADDRESS_SEPOLIA, URL,
};
use crate::helpers::runner::runner;
use crate::helpers::shell::os_specific_shell;
use camino::Utf8PathBuf;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;
use snapbox::cmd::cargo_bin;
use tempfile::tempdir;

#[tokio::test]
async fn test_happy_case_human_readable() {
    let tempdir = tempdir().unwrap();

    let calldata = r"NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }";

    let args = vec![
        "serialize",
        "--arguments",
        calldata,
        "--contract-address",
        DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "nested_struct_fn",
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(indoc! {r"
    Success: Serialization completed

    Calldata: [0x24, 0x60]
    "});
}

#[tokio::test]
async fn test_happy_case_json() {
    let tempdir = tempdir().unwrap();

    let calldata = r"NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }";

    let args = vec![
        "--json",
        "serialize",
        "--arguments",
        calldata,
        "--contract-address",
        DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "nested_struct_fn",
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(indoc! {r#"
        {"calldata":["0x24","0x60"]}
    "#});
}

#[tokio::test]
async fn test_contract_does_not_exist() {
    let args = vec![
        "serialize",
        "--arguments",
        "some_calldata",
        "--contract-address",
        "0x1",
        "--function",
        "nested_struct_fn",
        "--url",
        URL,
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: An error occurred in the called contract[..]Requested contract address[..]is not deployed[..]",
    );
}

#[tokio::test]
async fn test_wrong_function_name() {
    let calldata = r"NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }";

    let args = vec![
        "serialize",
        "--arguments",
        calldata,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "nonexistent_function",
        "--url",
        URL,
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        r#"Error: Failed to transform arguments into calldata
Caused by:
    Function with selector "0x38a013a14030cb08ae86482a9e0f3bad42daeb0222bdfe0634d77388deab9b9" not found in ABI of the contract"#,
    );
}

#[tokio::test]
async fn test_happy_case_shell() {
    let binary_path = cargo_bin!("sncast");
    let command = os_specific_shell(&Utf8PathBuf::from("tests/shell/serialize"));

    let snapbox = command
        .arg(binary_path)
        .arg(URL)
        .arg(DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA);
    snapbox.assert().success();
}
