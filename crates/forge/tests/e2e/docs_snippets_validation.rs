use clap::Parser;
use docs::validation::{
    assert_valid_snippet, extract_snippets_from_directory, get_parent_dir,
    print_skipped_snippet_message, print_success_message,
};
use forge::Cli;
use regex::Regex;

#[test]
fn test_docs_snippets() {
    let root_dir = get_parent_dir(2);
    let docs_dir = root_dir.join("docs/src");

    let re = Regex::new(r"(?ms)```shell\n\$ (snforge .+?)\n```").expect("Invalid regex pattern");

    let snippets = extract_snippets_from_directory(&docs_dir, &re)
        .expect("Failed to extract snforge command snippets");

    // TODO(#2684)
    let skipped_args = [
        // for some reason `try_parse_from` fails on `--version` flag
        vec!["snforge", "--version"],
    ];

    for snippet in &snippets {
        let args = snippet.to_command_args();
        let args: Vec<&str> = args.iter().map(String::as_str).collect();

        if skipped_args.contains(&args) {
            print_skipped_snippet_message(snippet, "snforge");
            continue;
        }

        let parse_result = Cli::try_parse_from(args);
        let err_message = if let Err(err) = &parse_result {
            err.to_string()
        } else {
            String::new()
        };
        assert_valid_snippet(parse_result.is_ok(), snippet, "snforge", &err_message);
    }

    print_success_message(snippets.len(), "snforge");
}
