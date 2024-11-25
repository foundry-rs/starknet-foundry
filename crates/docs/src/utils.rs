use std::{env, path::PathBuf};

use crate::snippet::Snippet;

#[must_use]
pub fn get_nth_ancestor(levels_up: usize) -> PathBuf {
    let mut dir = env::current_dir().expect("Failed to get the current directory");

    for _ in 0..levels_up {
        dir = dir
            .parent()
            .expect("Failed to navigate to parent directory")
            .to_owned();
    }

    dir
}

pub fn assert_valid_snippet(condition: bool, snippet: &Snippet, err_message: &str) {
    assert!(
        condition,
        "Found invalid {} snippet in the docs at {}:{}:1\n{}",
        snippet.snippet_type.as_str(),
        snippet.file_path,
        snippet.line_start,
        err_message
    );
}

pub fn print_success_message(snippets_len: usize, tool_name: &str) {
    println!("Successfully validated {snippets_len} {tool_name} docs snippets");
}

pub fn print_skipped_snippet_message(snippet: &Snippet) {
    println!(
        "Skipped validation of {} snippet in the docs in file: {} at line {}",
        snippet.snippet_type.as_str(),
        snippet.file_path,
        snippet.line_start,
    );
}
