use crate::helpers::{
    constants::CONTRACTS_DIR, fixtures::duplicate_contract_directory_with_salt, runner::runner,
};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

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
