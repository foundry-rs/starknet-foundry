use crate::helpers::constants::{SCRIPTS_DIR, URL};
use crate::helpers::fixtures::{duplicate_directory_and_salt_file, get_accounts_path};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};
use tempfile::TempDir;

fn duplicate_map_script(salt: &str) -> TempDir {
    duplicate_directory_and_salt_file(
        SCRIPTS_DIR.to_owned() + "/map_script",
        Some(SCRIPTS_DIR.to_owned()),
        "dummy",
        "contracts/src/lib.cairo",
        salt,
    )
}

#[tokio::test]
async fn test_default_verbosity() {
    let current_dir = duplicate_map_script("13");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let script_name = "map_script";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user3",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(current_dir.path().join("scripts"))
        .args(args);

    snapbox.assert().success().stdout_matches(indoc! {r"
           Compiling lib(map_script) map_script v0.1.0 [..]
           Compiling starknet-contract(map_script) map_script v0.1.0 [..]
            Finished release target(s) in [..] seconds


        cheatcode: declare
        class_hash: 0x[..]
        transaction_hash: 0x[..]

        cheatcode: deploy
        contract_address: 0x[..]
        transaction_hash: 0x[..]

        cheatcode: invoke
        transaction_hash: 0x[..]

        cheatcode: call
        response: [0x2]

        cheatcode: declare
        class_hash: 0x[..]
        transaction_hash: 0x[..]

        cheatcode: deploy
        contract_address: 0x[..]
        transaction_hash: 0x[..]

        cheatcode: invoke
        transaction_hash: 0x[..]

        cheatcode: call
        response: [0x3]

        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_quiet() {
    let current_dir = duplicate_map_script("14");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let script_name = "map_script";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user5",
        "--url",
        URL,
        "script",
        &script_name,
        "--quiet",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(current_dir.path().join("scripts"))
        .args(args);

    snapbox.assert().success().stdout_matches(indoc! {r"
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_one_of_the_steps_failing() {
    let current_dir = duplicate_map_script("15");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let script_name = "map_script_failing_step";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user6",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(current_dir.path().join("scripts"))
        .args(args);

    let result = snapbox.assert().success();

    let result = result.stdout_matches(indoc! {r"
           Compiling lib(map_script) map_script v0.1.0 [..]
           Compiling starknet-contract(map_script) map_script v0.1.0 [..]
            Finished release target(s) in [..] seconds


        cheatcode: declare
        class_hash: 0x[..]
        transaction_hash: 0x[..]

        cheatcode: deploy
        contract_address: 0x[..]
        transaction_hash: 0x[..]

    "});

    result.stderr_matches(indoc! {r#"
       Transaction hash: 0x[..]

       command: script
       error: Got an exception while executing a hint: Hint Error: Transaction has been reverted: Error in the called contract (0x[..]
       [..]
       Got an exception while executing a hint: Custom Hint Error: Entry point EntryPointSelector(StarkFelt("0x[..]
       ...
    "#});
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_verbose() {
    let current_dir = duplicate_map_script("16");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let script_name = "map_script_all_cheatcodes";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user7",
        "--url",
        URL,
        "script",
        &script_name,
        "--verbose",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(current_dir.path().join("scripts"))
        .args(args);

    snapbox.assert().success().stdout_matches(indoc! {r#"
           Compiling lib(map_script) map_script v0.1.0 [..]
           Compiling starknet-contract(map_script) map_script v0.1.0 [..]
            Finished release target(s) in [..] seconds


        cheatcode: declare
        args_passed: [
        	contract_name: "Mapa",
        	max_fee: Some(FieldElement { inner: 0x000000000000000000000000000000000000000000000000016345785d89ffff }),
        	nonce: Some(FieldElement { inner: 0x[..] }),
        ]

        cheatcode: declare
        class_hash: 0x[..]
        transaction_hash: 0x[..]

        cheatcode: deploy
        args_passed: [
        	class_hash: FieldElement { inner: 0x[..] },
        	constructor_calldata: [],
        	salt: Some(FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000003 }),
        	unique: true,
        	max_fee: Some(FieldElement { inner: 0x000000000000000000000000000000000000000000000000016345785d89ffff }),
        	nonce: Some(FieldElement { inner: 0x[..] }),
        ]

        cheatcode: deploy
        contract_address: 0x[..]
        transaction_hash: 0x[..]

        cheatcode: invoke
        args_passed: [
        	contract_address: FieldElement { inner: 0x[..] },
        	entry_point_name: "put",
        	calldata: [FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000001 }, FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000002 }],
        	max_fee: Some(FieldElement { inner: 0x000000000000000000000000000000000000000000000000016345785d89ffff }),
        	nonce: Some(FieldElement { inner: 0x[..] }),
        ]

        cheatcode: invoke
        transaction_hash: 0x[..]

        cheatcode: call
        args_passed: [
        	contract_address: FieldElement { inner: 0x[..] },
        	function_name: "get",
        	calldata_felts: [FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000001 }],
        ]

        cheatcode: call
        response: [0x2]

        command: script
        status: success
    "#});
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_verbose_with_json() {
    let current_dir = duplicate_map_script("17");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let script_name = "map_script_all_cheatcodes";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--url",
        URL,
        "--json",
        "script",
        &script_name,
        "--verbose",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(current_dir.path().join("scripts"))
        .args(args);

    snapbox.assert().success().stdout_matches(indoc! {r#"
        {"status":"compiling","message":"lib(map_script) map_script v0.1.0 [..]"}
        {"status":"compiling","message":"starknet-contract(map_script) map_script v0.1.0 [..]"}
        {"status":"finished","message":"release target(s) in [..] seconds"}


        {
          "args_passed": {
            "contract_name": "Mapa",
            "max_fee": "Some(FieldElement { inner: 0x000000000000000000000000000000000000000000000000016345785d89ffff })",
            "nonce": "Some(FieldElement { inner: 0x[..] })"
          },
          "cheatcode": "declare"
        }

        {
          "cheatcode": "declare",
          "class_hash": "0x[..]",
          "transaction_hash": "0x[..]"
        }

        {
          "args_passed": {
            "class_hash": "FieldElement { inner: 0x[..] }",
            "constructor_calldata": [],
            "max_fee": "Some(FieldElement { inner: 0x000000000000000000000000000000000000000000000000016345785d89ffff })",
            "nonce": "Some(FieldElement { inner: 0x[..] })",
            "salt": "Some(FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000003 })",
            "unique": true
          },
          "cheatcode": "deploy"
        }

        {
          "cheatcode": "deploy",
          "contract_address": "0x[..]",
          "transaction_hash": "0x[..]"
        }

        {
          "args_passed": {
            "calldata": "[FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000001 }, FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000002 }]",
            "contract_address": "FieldElement { inner: 0x[..] }",
            "entry_point_name": "put",
            "max_fee": "Some(FieldElement { inner: 0x000000000000000000000000000000000000000000000000016345785d89ffff })",
            "nonce": "Some(FieldElement { inner: 0x[..] })"
          },
          "cheatcode": "invoke"
        }

        {
          "cheatcode": "invoke",
          "transaction_hash": "0x[..]"
        }

        {
          "args_passed": {
            "calldata_felts": "[FieldElement { inner: 0x0000000000000000000000000000000000000000000000000000000000000001 }]",
            "contract_address": "FieldElement { inner: 0x[..] }",
            "function_name": "get"
          },
          "cheatcode": "call"
        }

        {
          "cheatcode": "call",
          "response": "[0x2]"
        }

        {
          "command": "script",
          "status": "success"
        }
    "#});
}
