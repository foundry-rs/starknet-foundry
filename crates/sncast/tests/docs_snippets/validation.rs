use std::collections::HashMap;
use std::fs;

use camino::Utf8PathBuf;
use docs::snippet::{Snippet, SnippetType};
use docs::utils::{
    get_nth_ancestor, print_skipped_snippet_message, print_success_message,
    update_scarb_toml_dependencies,
};
use docs::validation::{extract_snippets_from_directory, extract_snippets_from_file};
use regex::Regex;
use shared::test_utils::output_assert::{assert_stdout_contains, AsOutput};
use tempfile::TempDir;

use crate::helpers::constants::URL;
use crate::helpers::fixtures::copy_directory_to_tempdir;
use crate::helpers::runner::runner;

struct Contract {
    class_hash: String,
    contract_address: String,
}

fn prepare_accounts_file(temp: &TempDir) -> Utf8PathBuf {
    // Account from predeployed accounts in starknet-devnet-rs
    let accounts = r#"
    {
        "alpha-sepolia": {
            "user0": {
            "address": "0x6f4621e7ad43707b3f69f9df49425c3d94fdc5ab2e444bfa0e7e4edeff7992d",
            "deployed": true,
            "private_key": "0x0000000000000000000000000000000056c12e097e49ea382ca8eadec0839401",
            "public_key": "0x048234b9bc6c1e749f4b908d310d8c53dae6564110b05ccf79016dca8ce7dfac",
            "type": "open_zeppelin"
            }
        }
    }
    "#;

    let accounts_path = temp.path().join("accounts.json");
    fs::write(&accounts_path, accounts).expect("Failed to write accounts.json");

    Utf8PathBuf::from_path_buf(accounts_path).expect("Invalid UTF-8 path")
}

fn declare_and_deploy_contract(
    contract_name: &str,
    accounts_file: &str,
    temp: &TempDir,
) -> Contract {
    let args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        "user0",
        "declare",
        "--url",
        URL,
        "--contract-name",
        contract_name,
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "strk",
    ];

    let snapbox = runner(&args).current_dir(temp.path());
    let output = snapbox.assert().success();
    let re_class_hash = Regex::new(r"class_hash:\s+(0x[a-fA-F0-9]+)").unwrap();

    let class_hash = re_class_hash
        .captures(output.as_stdout())
        .and_then(|captures| captures.get(1))
        .map(|match_| match_.as_str())
        .expect("class_hash not found in the output");

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        "user0",
        "deploy",
        "--url",
        URL,
        "--class-hash",
        class_hash,
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "strk",
    ];

    let re_contract_address = Regex::new(r"contract_address:\s+(0x[a-fA-F0-9]+)").unwrap();

    let snapbox = runner(&args).current_dir(temp.path());
    let output = snapbox.assert().success();

    let contract_address = re_contract_address
        .captures(output.as_stdout())
        .and_then(|captures| captures.get(1))
        .map(|match_| match_.as_str())
        .expect("contract_address not found in the output");

    Contract {
        class_hash: class_hash.to_string(),
        contract_address: contract_address.to_string(),
    }
}

fn setup_contracts_map(
    tempdir: &TempDir,
    account_json_path: &Utf8PathBuf,
) -> HashMap<String, Contract> {
    let mut contracts: HashMap<String, Contract> = HashMap::new();
    let contract_names = [
        "HelloStarknet",
        "DataTransformerContract",
        "ConstructorContract",
    ];

    for contract_name in &contract_names {
        let contract =
            declare_and_deploy_contract(contract_name, account_json_path.as_str(), tempdir);
        contracts.insert((*contract_name).to_string(), contract);
    }

    contracts
}

fn swap_next_element<'a>(args: &mut [&'a str], target: &str, new_value: &'a str) {
    if let Some(index) = args.iter().position(|&x| x == target) {
        if index + 1 < args.len() {
            args[index + 1] = new_value;
        }
    }
}

fn is_command(args: &[&str], commands: &[&str]) -> bool {
    commands.iter().any(|&cmd| args.contains(&cmd))
}

#[test]
fn test_docs_snippets() {
    let root_dir_path: std::path::PathBuf = get_nth_ancestor(2);
    let docs_dir_path = root_dir_path.join("docs/src");
    let sncast_readme_path = root_dir_path.join("crates/sncast/README.md");

    let snippet_type = SnippetType::sncast();

    let docs_snippets = extract_snippets_from_directory(&docs_dir_path, &snippet_type)
        .expect("Failed to extract command snippets");

    let readme_snippets = extract_snippets_from_file(&sncast_readme_path, &snippet_type)
        .expect("Failed to extract command snippets");

    let snippets = docs_snippets
        .into_iter()
        .chain(readme_snippets)
        .collect::<Vec<Snippet>>();

    let sncast_example_dir =
        Utf8PathBuf::from_path_buf(root_dir_path.join("docs/listings/sncast_example"))
            .expect("Invalid UTF-8 path");
    let tempdir = copy_directory_to_tempdir(&sncast_example_dir);
    let accounts_json_path = prepare_accounts_file(&tempdir);

    update_scarb_toml_dependencies(&tempdir).unwrap();

    let contracts = setup_contracts_map(&tempdir, &accounts_json_path);

    for snippet in &snippets {
        if snippet.config.ignored.unwrap_or(false) {
            print_skipped_snippet_message(snippet);
            continue;
        }

        let args = snippet.to_command_args();
        let mut args: Vec<&str> = args.iter().map(String::as_str).collect();

        // remove "sncast" from the args
        args.remove(0);

        args.insert(0, "--accounts-file");
        args.insert(1, accounts_json_path.as_str());

        if let Some(contract_name) = &snippet.config.contract_name {
            let contract = contracts.get(contract_name).expect("Contract not found");

            // In case of invoke/call/verify, we need to replace contract address insnippet's
            // args with prepared contract's address
            if is_command(&args, &["invoke", "call", "verify"]) {
                swap_next_element(&mut args, "--contract-address", &contract.contract_address);
            // Similarly for deploy, we need to replace class-hash in snippet's
            // args with prepared contract's class-hash
            } else if is_command(&args, &["deploy"]) {
                swap_next_element(&mut args, "--class-hash", &contract.class_hash);
            }
        }

        let snapbox = runner(&args).current_dir(tempdir.path());
        let output = snapbox.assert().success();

        if snippet.output.is_some() && !snippet.config.ignored_output.unwrap_or(false) {
            assert_stdout_contains(output, snippet.output.as_ref().unwrap());
        }
    }

    print_success_message(&snippets, snippet_type.as_str());
}
