use crate::snippet::Snippet;
use camino::Utf8PathBuf;
use std::{env, fs, path::PathBuf, str::FromStr};
use tempfile::TempDir;
use toml_edit::{value, DocumentMut};

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

pub fn print_success_message(snippets: &[Snippet], tool_name: &str) {
    let validated_snippets_count = snippets
        .iter()
        .filter(|snippet| !snippet.config.ignored.unwrap_or(false)) // Filter out ignored snippets
        .count();
    let ignored_snippets_count = snippets.len() - validated_snippets_count;
    println!("Finished validation of {tool_name} docs snippets\nValidated: {validated_snippets_count}, Ignored: {ignored_snippets_count}");
}

pub fn print_skipped_snippet_message(snippet: &Snippet) {
    println!(
        "Ignoring {} docs snippet, file: {} :{}:1\n",
        snippet.snippet_type.as_str(),
        snippet.file_path,
        snippet.line_start,
    );
}

pub fn update_scarb_toml_dependencies(temp: &TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let snforge_std_path = Utf8PathBuf::from_str("../../snforge_std")
        .unwrap()
        .canonicalize_utf8()
        .unwrap()
        .to_string()
        .replace('\\', "/");

    let sncast_std_path = Utf8PathBuf::from_str("../../sncast_std")
        .unwrap()
        .canonicalize_utf8()
        .unwrap()
        .to_string()
        .replace('\\', "/");

    let scarb_toml_path = temp.path().join("Scarb.toml");
    let mut scarb_toml = fs::read_to_string(&scarb_toml_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();

    scarb_toml["dependencies"]["sncast_std"]["path"] = value(&sncast_std_path);
    scarb_toml["dev-dependencies"]["snforge_std"]["path"] = value(&snforge_std_path);

    fs::write(&scarb_toml_path, scarb_toml.to_string())?;

    Ok(())
}
