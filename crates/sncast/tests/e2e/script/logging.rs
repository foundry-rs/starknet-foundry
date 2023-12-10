use crate::helpers::constants::{SCRIPTS_DIR, URL};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};

#[tokio::test]
async fn test_default_verbosity() {
    let script_name = "map_script";
    let args = vec![
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/map_script/scripts")
        .args(args);

    snapbox.assert().success().stdout_matches(indoc! {r#"
           Compiling lib(map_script) map_script v0.1.0 [..]
           Compiling starknet-contract(map_script) map_script v0.1.0 [..]
            Finished release target(s) in [..] seconds


        Executing script "map_script"

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
    "#});
}

#[tokio::test]
async fn test_quiet() {
    let script_name = "map_script";
    let args = vec![
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        &script_name,
        "--quiet",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/map_script/scripts")
        .args(args);

    snapbox.assert().success().stdout_matches(indoc! {r"
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_one_of_the_steps_failing() {
    let script_name = "map_script_failing_step";
    let args = vec![
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/map_script/scripts")
        .args(args);

    let result = snapbox.assert().success();

    let result = result.stdout_matches(indoc! {r#"
           Compiling lib(map_script) map_script v0.1.0 [..]
           Compiling starknet-contract(map_script) map_script v0.1.0 [..]
            Finished release target(s) in [..] seconds


        Executing script "map_script_failing_step"

        cheatcode: declare
        class_hash: 0x[..]
        transaction_hash: 0x[..]

        cheatcode: deploy
        contract_address: 0x[..]
        transaction_hash: 0x[..]

    "#});

    result.stderr_matches(indoc! {r#"
       Transaction hash: 0x[..]

       command: script
       error: Got an exception while executing a hint: Hint Error: Transaction has been reverted: Error in the called contract (0x[..]
       [..]
       Got an exception while executing a hint: Hint Error: Entry point EntryPointSelector(StarkFelt("0x[..]
       ...
    "#});
}

#[tokio::test]
async fn test_verbose() {
    let script_name = "map_script";
    let args = vec![
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        &script_name,
        "--verbose",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/map_script/scripts")
        .args(args);

    snapbox.assert().success().stdout_matches(indoc! {r#"
           Compiling lib(map_script) map_script v0.1.0 [..]
           Compiling starknet-contract(map_script) map_script v0.1.0 [..]
            Finished release target(s) in [..] seconds


        Executing script "map_script"

        Args passed to "declare" cheatcode:
        {
        	contract_name = "Mapa",
        	max_fee = Some(
        	    FieldElement {
        	        inner: 0x000000000000000000000000000000000000000000000000016345785d89ffff,
        	    },
        	),
        	nonce = Some(
        	    FieldElement {
        	        inner: 0x0000000000000000000000000000000000000000000000000000000000000001,
        	    },
        	),
        }

        cheatcode: declare
        class_hash: 0x[..]
        transaction_hash: 0x[..]

        Args passed to "deploy" cheatcode:
        {
        	class_hash = FieldElement {
        	    inner: 0x[..],
        	},
        	constructor_calldata = [],
        	salt = Some(
        	    FieldElement {
        	        inner: 0x0000000000000000000000000000000000000000000000000000000000000003,
        	    },
        	),
        	unique = true,
        	max_fee = Some(
        	    FieldElement {
        	        inner: 0x000000000000000000000000000000000000000000000000016345785d89ffff,
        	    },
        	),
        	nonce = Some(
        	    FieldElement {
        	        inner: 0x0000000000000000000000000000000000000000000000000000000000000002,
        	    },
        	),
        }

        cheatcode: deploy
        contract_address: 0x[..]
        transaction_hash: 0x[..]

        Args passed to "invoke" cheatcode:
        {
        	contract_address = FieldElement {
        	    inner: 0x[..],
        	},
        	entry_point_name = "put",
        	calldata = [
        	    FieldElement {
        	        inner: 0x0000000000000000000000000000000000000000000000000000000000000001,
        	    },
        	    FieldElement {
        	        inner: 0x0000000000000000000000000000000000000000000000000000000000000002,
        	    },
        	],
        	max_fee = Some(
        	    FieldElement {
        	        inner: 0x000000000000000000000000000000000000000000000000016345785d89ffff,
        	    },
        	),
        	nonce = Some(
        	    FieldElement {
        	        inner: 0x0000000000000000000000000000000000000000000000000000000000000003,
        	    },
        	),
        }

        cheatcode: invoke
        transaction_hash: 0x[..]

        Args passed to "call" cheatcode:
        {
        	contract_address = FieldElement {
        	    inner: 0x[..],
        	},
        	function_name = "get",
        	calldata_felts = [
        	    FieldElement {
        	        inner: 0x0000000000000000000000000000000000000000000000000000000000000001,
        	    },
        	],
        }

        cheatcode: call
        response: [0x2]

        ...
    "#});
}
