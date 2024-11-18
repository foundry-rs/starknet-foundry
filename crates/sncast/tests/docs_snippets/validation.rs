use docs::validation::{
    assert_valid_snippet, extract_snippets_from_directory, extract_snippets_from_file,
    get_parent_dir, print_success_message, Snippet,
};
use regex::Regex;
use tempfile::tempdir;

use crate::helpers::runner::runner;

#[test]
fn test_docs_snippets() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");

    let root_dir_path = get_parent_dir(2);
    let docs_dir_path = root_dir_path.join("docs/src");
    let sncast_readme_path = root_dir_path.join("crates/sncast/README.md");

    let re = Regex::new(r"(?ms)```shell\n\$ (sncast .+?)\n```").expect("Invalid regex pattern");
    let extension = Some("md");
    let docs_snippets = extract_snippets_from_directory(&docs_dir_path, &re, extension)
        .expect("Failed to extract sncast command snippets");

    let readme_snippets = extract_snippets_from_file(&sncast_readme_path, &re)
        .expect("Failed to extract sncast command snippets");

    let snippets = docs_snippets
        .into_iter()
        .chain(readme_snippets)
        .collect::<Vec<Snippet>>();

    let skipped_args = [
        // snippet "$ sncast <subcommand>"
        vec!["<subcommand>"],
        // snippet with interactive account import example
        vec![
            "account",
            "import",
            "--url",
            "http://127.0.0.1:5050",
            "--name",
            "account_123",
            "--address",
            "0x1",
            "--type",
            "oz",
        ],
    ];

    for snippet in &snippets {
        let args = snippet.to_command_args();
        let mut args: Vec<&str> = args.iter().map(String::as_str).collect();
        args.remove(0);

        if skipped_args.contains(&args) {
            continue;
        }

        // TODO(#2678)

        let snapbox = runner(&args).current_dir(tempdir.path());
        let output = snapbox.output().expect("Failed to execute the command");
        let exit_code = output.status.code().unwrap_or_default();
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert_valid_snippet(exit_code != 2, snippet, "sncast", &stderr);
    }

    print_success_message(snippets.len(), "sncast");
}
