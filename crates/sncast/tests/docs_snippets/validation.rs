use crate::helpers::runner::runner;
use docs::validation::{
    extract_matches_from_directory, extract_matches_from_file, get_parent_dir,
    parse_snippet_str_to_command_args,
};
use regex::Regex;
use tempfile::tempdir;

#[test]
fn test_docs_snippets() {
    let tempdir: tempfile::TempDir = tempdir().expect("Unable to create a temporary directory");
    let root_dir_path = get_parent_dir(2);
    let docs_dir_path = root_dir_path.join("docs/src");
    let sncast_readme_path = root_dir_path.join("crates/sncast/README.md");

    let re = Regex::new(r"(?ms)```shell\n\$ sncast(.+?)\n```").expect("Invalid regex pattern");
    let extension = Some("md");
    let docs_snippets = extract_matches_from_directory(&docs_dir_path, &re, extension)
        .expect("Failed to extract sncast command snippets");

    println!("sncast_readme_path: {:?}", sncast_readme_path);
    let readme_snippets = extract_matches_from_file(&sncast_readme_path, &re)
        .expect("Failed to extract sncast command snippets");

    let snippets = docs_snippets
        .into_iter()
        .chain(readme_snippets.into_iter())
        .collect::<Vec<String>>();

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

    for snippet in snippets.clone() {
        let args = parse_snippet_str_to_command_args(snippet.as_str());
        let args: Vec<&str> = args.iter().map(String::as_str).collect();

        if skipped_args.contains(&args) {
            continue;
        }

        let snapbox = runner(&args).current_dir(tempdir.path());
        let output = snapbox.output().expect("Failed to execute the command");
        let exit_code = output.status.code().unwrap_or_default();
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert_ne!(
            exit_code, 2,
            "The command {snippet} failed. Stderr: {stderr}"
        );
    }

    println!(
        "Validated {} sncast command snippets in the docs",
        snippets.len()
    );
}
