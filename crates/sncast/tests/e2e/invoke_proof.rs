use crate::helpers::constants::{ACCOUNT_FILE_PATH, MAP_CONTRACT_ADDRESS_SEPOLIA, URL};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;

// TODO: Add full tests once proof generation flow is possible.

#[test]
fn test_proof_file_requires_proof_facts_file() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user11",
        "invoke",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x1",
        "0x2",
        "--proof-file",
        "proof.txt",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert();

    assert_stderr_contains(
        output,
        indoc! {r"
        error: the following required arguments were not provided:
          --proof-facts-file <PROOF_FACTS_FILE>
        "},
    );
}

#[test]
fn test_proof_facts_file_requires_proof_file() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        "user11",
        "invoke",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x1",
        "0x2",
        "--proof-facts-file",
        "proof-facts.txt",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert();

    assert_stderr_contains(
        output,
        indoc! {r"
        error: the following required arguments were not provided:
          --proof-file <PROOF_FILE>
        "},
    );
}
