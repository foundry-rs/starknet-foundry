use docs::snippet::{Snippet, SnippetType};
use docs::utils::{
    assert_valid_snippet, get_parent_dir, print_skipped_snippet_message, print_success_message,
};
use docs::validation::{extract_snippets_from_directory, extract_snippets_from_file};
use tempfile::tempdir;

use crate::helpers::runner::runner;

#[test]
fn test_docs_snippets() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");

    let root_dir_path = get_parent_dir(2);
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

    for snippet in &snippets {
        let args = snippet.to_command_args();
        let mut args: Vec<&str> = args.iter().map(String::as_str).collect();

        // remove "sncast" from the args
        args.remove(0);

        if snippet.config.ignored.unwrap_or(false) {
            print_skipped_snippet_message(snippet);
            continue;
        }

        let snapbox = runner(&args).current_dir(tempdir.path());
        let output = snapbox.output().expect("Failed to execute the command");
        let exit_code = output.status.code().unwrap_or_default();
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert_valid_snippet(exit_code != 2, snippet, &stderr);
    }

    print_success_message(snippets.len(), snippet_type.as_str());
}
