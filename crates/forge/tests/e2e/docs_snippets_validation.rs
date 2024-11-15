use clap::Parser;
use docs::validation::{extract_matches_from_directory, get_parent_dir, snippet_to_command_args};
use forge::Cli;
use regex::Regex;
#[test]
fn test_docs_snippets() {
    let root_dir = get_parent_dir(2);
    let docs_dir = root_dir.join("docs/src");

    let re = Regex::new(r"(?ms)```shell\n\$ (snforge .+?)\n```").expect("Invalid regex pattern");
    let extension = Some("md");

    let snippets = extract_matches_from_directory(&docs_dir, &re, extension)
        .expect("Failed to extract snforge command snippets");

    let skipped_args = [
        // for some reason `try_parse_from` fails on `--version` flag
        vec!["snforge", "--version"],
    ];

    for snippet in snippets {
        let args = snippet_to_command_args(snippet.as_str());
        let args: Vec<&str> = args.iter().map(String::as_str).collect();

        if skipped_args.contains(&args) {
            continue;
        }

        let parse_result = Cli::try_parse_from(args);

        assert!(
            parse_result.is_ok(),
            "Found invalid snforge snippet in the docs: {:?}\n{}",
            snippet,
            parse_result.err().unwrap()
        );
    }
}
