use clap::Parser;
use docs::snippet::SnippetType;
use docs::utils::{
    assert_valid_snippet, get_nth_ancestor, print_skipped_snippet_message, print_success_message,
};
use docs::validation::extract_snippets_from_directory;
use forge::Cli;
use shared::test_utils::output_assert::assert_stdout_contains;

use super::common::runner::{runner, setup_package};

#[test]
fn test_docs_snippets() {
    let root_dir = get_nth_ancestor(2);
    let docs_dir = root_dir.join("docs/src");

    let snippet_type = SnippetType::forge();

    let snippets = extract_snippets_from_directory(&docs_dir, &snippet_type)
        .expect("Failed to extract command snippets");

    for snippet in &snippets {
        let args = snippet.to_command_args();
        let mut args: Vec<&str> = args.iter().map(String::as_str).collect();

        if snippet.config.ignored.unwrap_or(false) {
            print_skipped_snippet_message(snippet);
            continue;
        }

        let parse_result = Cli::try_parse_from(args.clone());
        let err_message = if let Err(err) = &parse_result {
            err.to_string()
        } else {
            String::new()
        };

        assert_valid_snippet(parse_result.is_ok(), snippet, &err_message);

        // Remove "snforge" from the args
        args.remove(0);

        if let Some(snippet_output) = &snippet.output {
            let package_name = snippet
                .config
                .package_name
                .clone()
                .or_else(|| snippet.capture_package_from_output())
                .expect("Cannot find package name in command output or snippet config");

            let temp = setup_package(&package_name);
            let output = runner(&temp).args(args).assert();

            assert_stdout_contains(output, snippet_output);
        }
    }

    print_success_message(&snippets, snippet_type.as_str());
}
