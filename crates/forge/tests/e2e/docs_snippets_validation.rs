use crate::e2e::common::runner::{setup_package, test_runner};
use docs::validation::{
    extract_matches_from_directory, get_parent_dir, parse_snippet_str_to_command_args,
};
use regex::Regex;

#[test]
fn test_docs_snippets() {
    let temp = setup_package("erc20_package");
    let root_dir = get_parent_dir(2);
    let docs_dir = root_dir.join("docs/src");

    let re = Regex::new(r"(?ms)```shell\n\$ snforge(.+?)\n```").expect("Invalid regex pattern");

    let extension = Some("md");
    let snippets = extract_matches_from_directory(&docs_dir, &re, extension)
        .expect("Failed to extract snforge command snippets");

    for snippet in snippets.clone() {
        println!("SNIPPET: {snippet}");
        let args = parse_snippet_str_to_command_args(snippet.as_str());
        let args: Vec<&str> = args.iter().map(String::as_str).collect();

        let snapbox = test_runner(&temp).args(args);
        let output = snapbox.output().expect("Failed to execute the command");
        let exit_code = output.status.code().unwrap_or_default();
        let stderr = String::from_utf8_lossy(&output.stderr);

        // TODO: Change logic of validating forge commands
        assert_ne!(
            exit_code, 2,
            "The command {snippet} failed. Stderr: {stderr}"
        );
    }

    println!(
        "Validated {} snforge command snippets in the docs",
        snippets.len()
    );
}
