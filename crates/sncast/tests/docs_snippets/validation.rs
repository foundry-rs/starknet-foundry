use crate::helpers::devnet::{prepare_accounts_file, setup_contracts_map};
use crate::helpers::fixtures::copy_directory_to_tempdir;
use crate::helpers::runner::runner;
use camino::Utf8PathBuf;
use docs::snippet::{Snippet, SnippetType};
use docs::utils::{
    get_nth_ancestor, print_ignored_snippet_message, print_snippets_validation_summary,
    update_scarb_toml_dependencies,
};
use docs::validation::{extract_snippets_from_directory, extract_snippets_from_file};
use shared::test_utils::output_assert::assert_stdout_contains;

fn swap_next_element<'a>(args: &mut [&'a str], target: &str, new_value: &'a str) {
    if let Some(index) = args.iter().position(|&x| x == target) {
        if index + 1 < args.len() {
            args[index + 1] = new_value;
        }
    }
}

fn get_contract_name_from_args(args: &[&str]) -> Option<String> {
    let index = args.iter().position(|&x| x == "--contract-name")?;
    args.get(index + 1).copied().map(String::from)
}

fn is_command(args: &[&str], commands: &[&str]) -> bool {
    commands.iter().any(|&cmd| args.contains(&cmd))
}

#[test]
fn test_docs_snippets() {
    let root_dir_path = get_nth_ancestor(2);
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

    let hello_sncast_dir =
        Utf8PathBuf::from_path_buf(root_dir_path.join("docs/listings/hello_sncast"))
            .expect("Invalid UTF-8 path");
    let tempdir = copy_directory_to_tempdir(&hello_sncast_dir);
    let accounts_json_path = prepare_accounts_file(&tempdir);

    update_scarb_toml_dependencies(&tempdir).unwrap();

    let contracts = setup_contracts_map(&tempdir, &accounts_json_path);

    for snippet in &snippets {
        if snippet.config.ignored {
            print_ignored_snippet_message(snippet);
            continue;
        }

        let args = snippet.to_command_args();
        let mut args: Vec<&str> = args.iter().map(String::as_str).collect();

        // remove "sncast" from the args
        args.remove(0);

        args.insert(0, "--accounts-file");
        args.insert(1, accounts_json_path.as_str());

        if let Some(contract_name) =
            get_contract_name_from_args(&args).or_else(|| snippet.config.contract_name.clone())
        {
            let contract = contracts
                .get(contract_name.as_str())
                .unwrap_or_else(|| panic!("Contract {contract_name} not found"));

            // In case of invoke/call/verify, we need to replace contract address in snippet's
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

        if snippet.output.is_some() && !snippet.config.ignored_output {
            assert_stdout_contains(output, snippet.output.as_ref().unwrap());
        }
    }

    print_snippets_validation_summary(&snippets, snippet_type.as_str());
}
