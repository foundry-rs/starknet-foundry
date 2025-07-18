use crate::helpers::constants::{
    DATA_TRANSFORMER_CONTRACT_ABI_PATH, DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA,
    DATA_TRANSFORMER_CONTRACT_CLASS_HASH_SEPOLIA, MAP_CONTRACT_ADDRESS_SEPOLIA, URL,
};
use crate::helpers::runner::runner;
use crate::helpers::shell::os_specific_shell;
use camino::Utf8PathBuf;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;
use snapbox::cmd::cargo_bin;
use tempfile::tempdir;

#[tokio::test]
async fn test_happy_case() {
    let tempdir = tempdir().unwrap();

    let calldata = r"NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }";

    let args = vec![
        "utils",
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
    Calldata: [0x24, 0x60]
    "});
}

#[tokio::test]
async fn test_happy_case_class_hash() {
    let tempdir = tempdir().unwrap();

    let calldata = r"NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }";

    let args = vec![
        "utils",
        "serialize",
        "--arguments",
        calldata,
        "--class-hash",
        DATA_TRANSFORMER_CONTRACT_CLASS_HASH_SEPOLIA,
        "--function",
        "nested_struct_fn",
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(indoc! {r"
    Calldata: [0x24, 0x60]
    "});
}

#[tokio::test]
async fn test_happy_case_abi_file() {
    let tempdir = tempdir().unwrap();
    let abi_file_path = Utf8PathBuf::from(DATA_TRANSFORMER_CONTRACT_ABI_PATH);
    let temp_abi_file_path = tempdir.path().join(abi_file_path.file_name().unwrap());
    std::fs::copy(abi_file_path, &temp_abi_file_path)
        .expect("Failed to copy ABI file to temp directory");

    let calldata = r"NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }";

    let args = vec![
        "utils",
        "serialize",
        "--arguments",
        calldata,
        "--abi-file",
        temp_abi_file_path.to_str().unwrap(),
        "--function",
        "nested_struct_fn",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(indoc! {r"
    Calldata: [0x24, 0x60]
    "});
}

#[tokio::test]
async fn test_abi_file_missing_function() {
    let tempdir = tempdir().unwrap();
    let abi_file_path =
        Utf8PathBuf::from("tests/data/files/data_transformer_contract_abi_missing_function.json");
    let temp_abi_file_path = tempdir.path().join(abi_file_path.file_name().unwrap());
    std::fs::copy(abi_file_path, &temp_abi_file_path)
        .expect("Failed to copy ABI file to temp directory");

    let calldata = r"NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }";

    let args = vec![
        "utils",
        "serialize",
        "--arguments",
        calldata,
        "--abi-file",
        temp_abi_file_path.to_str().unwrap(),
        "--function",
        "nested_struct_fn",
    ];

    let output = runner(&args).current_dir(tempdir.path()).assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r#"
    Error: Function with selector "0x2cf7c96d7437a80a891adac280b9089dbe00c5413e7d253bbc87845271ae772" not found in ABI of the contract
    "#},
    );
}

#[tokio::test]
async fn test_abi_file_missing_type() {
    let tempdir = tempdir().unwrap();
    let abi_file_path =
        Utf8PathBuf::from("tests/data/files/data_transformer_contract_abi_missing_type.json");
    let temp_abi_file_path = tempdir.path().join(abi_file_path.file_name().unwrap());
    std::fs::copy(abi_file_path, &temp_abi_file_path)
        .expect("Failed to copy ABI file to temp directory");

    let calldata = r"NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }";

    let args = vec![
        "utils",
        "serialize",
        "--arguments",
        calldata,
        "--abi-file",
        temp_abi_file_path.to_str().unwrap(),
        "--function",
        "nested_struct_fn",
    ];

    let output = runner(&args).current_dir(tempdir.path()).assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r#"
    Error: Error while processing Cairo-like calldata
        Struct "NestedStructWithField" not found in ABI
    "#},
    );
}

#[tokio::test]
async fn test_happy_case_json() {
    let tempdir = tempdir().unwrap();

    let calldata = r"NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }";

    let args = vec![
        "--json",
        "utils",
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
        "utils",
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
        "utils",
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
        r#"Error: Function with selector "0x38a013a14030cb08ae86482a9e0f3bad42daeb0222bdfe0634d77388deab9b9" not found in ABI of the contract"#,
    );
}

#[tokio::test]
async fn test_rpc_args_not_passed_when_using_class_hash() {
    let tempdir = tempdir().unwrap();

    let calldata = r"NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }";

    let args = vec![
        "utils",
        "serialize",
        "--arguments",
        calldata,
        "--class-hash",
        DATA_TRANSFORMER_CONTRACT_CLASS_HASH_SEPOLIA,
        "--function",
        "nested_struct_fn",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().failure().stderr_eq(indoc! {r"
    Error: Either `--network` or `--url` must be provided when using `--class-hash`
    "});
}

#[tokio::test]
async fn test_rpc_args_not_passed_when_using_contract_address() {
    let tempdir = tempdir().unwrap();

    let calldata = r"NestedStructWithField { a: SimpleStruct { a: 0x24 }, b: 96 }";

    let args = vec![
        "utils",
        "serialize",
        "--arguments",
        calldata,
        "--contract-address",
        DATA_TRANSFORMER_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "nested_struct_fn",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().failure().stderr_eq(indoc! {r"
    Error: Either `--network` or `--url` must be provided when using `--contract-address`
    "});
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
