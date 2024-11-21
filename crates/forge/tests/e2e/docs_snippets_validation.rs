use std::collections::HashMap;

use clap::Parser;
use docs::validation::{
    assert_valid_snippet, create_listings_to_packages_mapping, extract_snippets_from_directory,
    get_parent_dir, print_skipped_snippet_message, print_success_message, SnippetType,
};
use forge::Cli;
use shared::test_utils::output_assert::assert_stdout_contains;

use super::common::runner::{
    setup_hello_workspace, setup_package, setup_package_from_docs_listings, test_runner,
};

fn is_package_from_docs_listings(
    package: &str,
    listings_to_packages_mapping: &HashMap<String, Vec<String>>,
) -> bool {
    for packages in listings_to_packages_mapping.values() {
        if packages.contains(&package.to_string()) {
            return true;
        }
    }
    false
}

#[test]
fn test_docs_snippets() {
    let listings_to_packages_mapping = create_listings_to_packages_mapping();

    let root_dir = get_parent_dir(2);
    let docs_dir = root_dir.join("docs/src");

    let snippet_type = SnippetType::Forge;

    let snippets = extract_snippets_from_directory(&docs_dir, &snippet_type)
        .expect("Failed to extract snforge command snippets");

    // TODO(#2684)
    let skipped_args = [
        // For some reason `try_parse_from` fails on `--version` flag, it returns Err but produces the expected output
        vec!["snforge", "--version"],
    ];

    for snippet in &snippets {
        let args = snippet.to_command_args();
        let mut args: Vec<&str> = args.iter().map(String::as_str).collect();

        if skipped_args.contains(&args) {
            print_skipped_snippet_message(snippet, snippet_type.as_str());
            continue;
        }

        let parse_result = Cli::try_parse_from(args.clone());
        let err_message = if let Err(err) = &parse_result {
            err.to_string()
        } else {
            String::new()
        };

        assert_valid_snippet(parse_result.is_ok(), snippet, "snforge", &err_message);

        args.retain(|element| element != &"snforge" && element != &"test");

        if let Some(snippet_output) = &snippet.output {
            let package = snippet
                .capture_package_from_output()
                .expect("Failed to capture package from command output");

            let temp = if is_package_from_docs_listings(&package, &listings_to_packages_mapping) {
                setup_package_from_docs_listings(&package, &listings_to_packages_mapping)
            } else {
                let package = if ["addition", "fibonacci"].contains(&package.as_str()) {
                    "hello_workspaces"
                } else {
                    &package
                };
                if package == "hello_workspaces" {
                    setup_hello_workspace()
                } else {
                    setup_package(package)
                }
            };
            let output = test_runner(&temp).args(args).assert();

            assert_stdout_contains(output, snippet_output);
        }
    }

    print_success_message(snippets.len(), snippet_type.as_str());
}
