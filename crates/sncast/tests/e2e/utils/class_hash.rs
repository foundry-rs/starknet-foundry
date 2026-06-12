use crate::helpers::{
    constants::CONTRACTS_DIR,
    fixtures::{copy_directory_to_tempdir, duplicate_contract_directory_with_salt},
    runner::runner,
};
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};

#[test]
fn test_happy_case_get_class_hash() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "human_readable",
    );

    let args = vec!["utils", "class-hash", "--contract-name", "Map"];

    let snapbox = runner(&args).current_dir(contract_path.path());

    let output = snapbox.assert().success();

    assert_stdout_contains(output, indoc! {r"Class Hash: 0x0[..]"});
}

#[test]
fn test_errors_on_ambiguous_contract_name() {
    let contract_path =
        copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/duplicate_contract_name");

    let args = vec!["utils", "class-hash", "--contract-name", "HelloStarknet"];

    let output = runner(&args)
        .current_dir(contract_path.path())
        .assert()
        .failure();

    assert_stderr_contains(
        output,
        indoc! {r#"
        Error: Found more than one contract named "HelloStarknet" in artifacts, pass one of the absolute module tree paths to `--contract-name`: duplicate_contract_name::first_contract::HelloStarknet, duplicate_contract_name::second_contract::HelloStarknet
        "#},
    );
}

#[test]
fn test_accepts_full_module_path_for_ambiguous_contract_name() {
    let contract_path =
        copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/duplicate_contract_name");

    let args = vec![
        "utils",
        "class-hash",
        "--contract-name",
        "duplicate_contract_name::first_contract::HelloStarknet",
    ];

    let output = runner(&args)
        .current_dir(contract_path.path())
        .assert()
        .success();

    assert_stdout_contains(output, indoc! {r"Class Hash: 0x0[..]"});
}
